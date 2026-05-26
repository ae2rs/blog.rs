---
title: 'Broomfilter: making Rust as fast as C++'
published: 2026-04-12
draft: false
---

# This post's origin story

## Why bloom filters?

I'm a Rust engineer at [amo](https://amo.co/) (we're [hiring](https://amo.co/jobs/) btw!), and we interview candidates for backend roles. During these interviews, we like to ask a few general-knowledge questions, and one of them is : can you explain what a bloom filter is?

Most people have a good sense of what a bloom filter is, but let's ask our dear friend [Le Chat](https://chat.mistral.ai/chat):

> A Bloom filter is a space-efficient probabilistic data structure used to test whether an element is a member of a set
>
> Key characteristics
>
> - Space-efficient: Uses much less memory than a hash table or a list.
> - Fast: Both insertion and lookup operations are O(k), where k is the number of hash functions used.
> - No false negatives: If the filter says an item is not in the set, it is definitely not there.
> - Possible false positives: If the filter says an item is in the set, there is a small probability it might not be.
>
> -- [Le Chat](https://chat.mistral.ai/chat), 2026

This is all well and good, but it's also very, very vague.

- Yes, most people know that it is probabilistic in the sense that it *can* return false positives.

- Yes, some people know that it involves a big array of bits and a hash function.

- Yes, a few people know that it uses *multiple* hash functions, and that tuning how many of them you use is how you balance memory usage against false positive rate.

But almost nobody who isn't neck-deep in data structures can explain *why a naïve bloom filter is actually slow in practice*, or what you can do about it.

## Ragebait and FFMPEG

On the sacred day of the 1st of April, I came across [this tweet](https://x.com/ffmpeg/status/2039115531744334180) from the official FFmpeg account. And I have to admit, I was hurt.

> FFmpeg is moving to Rust 🦀
>
> Our use of C and Assembly in FFmpeg has been an unacceptable violation of safety.
>
> FFmpeg will be running 10x slower, but we're doing it for your safety.
>
> All your videos will appear green. Safety first, working software later.

April Fool's. Obviously. But the thing about a joke is that it only works if the premise lands. And the premise (*Rust is inherently slow*) clearly landed for a lot of people. It bothered me. Not because the FFmpeg team was being mean, but because I'd seen enough unoptimized Rust in the wild to worry they might be... not entirely wrong.

So I built [broomfilter](https://crates.io/crates/broomfilter). A bloom filter, in Rust. And then I spent way too long making it fast.

Here's what I learned.

## What a bloom filter actually *is*

Let me explain this properly, because the Le Chat definition up top, while correct, tells you absolutely nothing useful.

A bloom filter is built on two things: a big array of bits (all starting at 0), and a set of `k` hash functions.

**Inserting** an item runs it through all `k` hash functions. Each one produces a position in the bit array. You set those `k` bits to 1.

**Querying** an item runs it through the same `k` hash functions, getting the same `k` positions, and checks whether all those bits are 1.

- If any bit is 0, the item was definitely never inserted. A bit can only go from 0 to 1, never the other way, so a 0 is conclusive.
- If all bits are 1, the item was *probably* inserted. But not certainly.

That last point is where the "probabilistic" part comes in. Say you insert "apple" and it sets bits 1, 4, and 7. Later you insert "banana", which sets bits 2, 4, and 9. Now you query "cherry", which hashes to positions 1, 2, and 7. All three happen to be set, not because you inserted "cherry", but because other items set those bits incidentally. That's a **false positive**.

False positives get more likely as the filter fills up. More items inserted means more bits set to 1, means more coincidental overlap. The math gives you roughly `(1 - e^{-kn/m})^k` for a filter of `m` bits with `k` functions and `n` inserted items. You don't need to memorize this. Just know that **more bits = fewer false positives**, and there is a provably optimal `k` for any given memory budget.

**False negatives are impossible by construction.** If you inserted something, its bits were set. They stay set.

## Problem 1: cache lines

Here's the thing most bloom filter implementations get wrong.

Modern CPUs have layered memory caches:
- **L1**: ~32 KB, ~1 ns to access
- **L2**: ~256 KB–1 MB, ~5 ns
- **L3**: up to 32 MB, ~20 ns
- **RAM**: ~100 ns

And crucially: data doesn't move byte-by-byte between these layers. It moves in **64-byte chunks** called cache lines. If you ask for a single bit, the CPU fetches the entire 64 bytes around it, whether you wanted them or not.

Now picture a standard bloom filter over a 1 MB bit array. Your `k = 14` hash functions point to 14 random positions scattered across that array. Each one is probably in a completely different cache line, in a completely different part of RAM. So one lookup costs up to **14 cache misses**, meaning up to 1,400 ns of just waiting for memory.

For a structure you're using specifically to make things fast, that's pretty embarrassing.

> **The elevator analogy.** You need to check 14 boxes, but they're spread across 14 different floors of a building. You ride the elevator 14 times. Even if each ride only takes a few seconds, it adds up fast, especially when you're doing millions of lookups.

The fix is called a **cache-line blocked bloom filter**.

Instead of one flat bit array, you divide it into 64-byte **blocks** (8 × 64-bit words = 512 bits per block). The first hash determines *which block* to look at. All `k` bit probes land inside that one block.

One block = one cache line. One lookup = one cache miss, maximum.

The trade-off: you're cramming all your bit probes into a 512-bit space instead of letting them roam freely across millions of bits. This means some bits unavoidably overlap more than they would otherwise, which increases the false positive rate slightly. For most use cases (caches, deduplication, IP blocklists) this is completely acceptable.

In broomfilter, `BlockedFilter` divides its backing array into blocks of 8 × `u64`. A hash selects the block; `probe_masks` distributes the `k` bits inside it.

## Problem 2: division is surprisingly expensive

To select which block an item belongs to, the obvious approach is `hash % num_blocks`.

Division. That's 20–40 CPU cycles on modern hardware.

The trick: if `num_blocks` is a power of 2 (which we can always arrange by rounding up during allocation), then `hash % num_blocks` is exactly equivalent to `hash & (num_blocks - 1)`. A bitwise AND is 1 cycle.

We precompute this in the constructor:

```rust
let block_mask = num_blocks
    .is_power_of_two()
    .then_some(num_blocks - 1)
    .unwrap_or(0);
```

At lookup time, one branch decides which path to take:

```rust
let block_idx = if self.block_mask != 0 {
    h1 & self.block_mask   // 1 cycle
} else {
    h1 % self.num_blocks   // 20–40 cycles
};
```

One branch, paid once per lookup. The inner hot loop never sees it.

## Problem 3: checking bits one at a time

Once we've selected the block and computed which bits to check, the scalar version does this:

```rust
for (j, &mask) in masks.iter().enumerate() {
    if *block_ptr.add(j) & mask != mask {
        return false;
    }
}
```

Eight words, eight comparisons, one after another.

This is where **SIMD** comes in.

SIMD stands for "single instruction, multiple data": a class of CPU instructions that operate on several values at once. On ARM chips (including all Apple Silicon), the SIMD extension is called **NEON**. NEON registers are 128-bit wide, holding 2 × 64-bit integers side by side.

Our block is 8 words = 4 NEON registers. Instead of 8 sequential checks, we load all 8 words at once and check them in 4 parallel operations.

The key instruction is `vbicq_u64(mask, block)`, which ARM calls "bit clear": it returns the bits that are in the mask but *absent* from the block. The bits we expected to find but didn't. We run this on all 4 pairs and OR the results together. If everything is zero, all required bits were present:

```rust
let missing = vorrq_u64(
    vorrq_u64(vbicq_u64(m0, b0), vbicq_u64(m1, b1)),
    vorrq_u64(vbicq_u64(m2, b2), vbicq_u64(m3, b3)),
);
(vgetq_lane_u64::<0>(missing) | vgetq_lane_u64::<1>(missing)) == 0
```

Measured on Apple M4, this is **17–35% faster** than the scalar loop for `contains`, depending on the scenario.

## The time I tried to be too clever

After the NEON win, I looked at the profiler. The `probe_masks` function (the step that *computes* which bits to check) was now the dominant cost. Obvious next move: vectorize that too.

The loop looks straightforward enough:

```rust
let step = h1.wrapping_mul(0x9E3779B97F4A7C15) | 1;
for i in 0..k {
    let bit = h2.wrapping_add(i.wrapping_mul(step)) & (BLOCK_BITS - 1);
    masks[(bit >> 6) as usize] |= 1 << (bit & 63);
}
```

The multiply-add part (`h2 + i * step`) can trivially run 4 iterations at once with NEON. So I wrote a NEON version that computed 4 bit positions per round:

```rust
// Compute h2 + i*step for 4 values of i at once.
// Each 128-bit register holds 2 u64 lanes:
//   base       = [h2 + i*step, h2 + (i+1)*step]
//   increments = [i+2,         i+3            ]
//   steps      = [step,        step           ]
let offsets = vaddq_u64(base, vmulq_u64(increments, steps));

// Mask to the 512-bit block range to get target bit positions.
let bits = vandq_u64(offsets, vdupq_n_u64(BLOCK_BITS - 1));

// Here the parallelism ends.
// bit >> 6 gives the word index, bit & 63 gives the bit within it.
// The word index is runtime-determined, so we must extract each
// lane and write sequentially. This is the scatter — no NEON for it.
let bit0 = vgetq_lane_u64::<0>(bits);
masks[(bit0 >> 6) as usize] |= 1 << (bit0 & 63);

let bit1 = vgetq_lane_u64::<1>(bits);
masks[(bit1 >> 6) as usize] |= 1 << (bit1 & 63);

// ... repeat for next pair, advance i by 4
```

The problem is the last line of each group: `masks[(bit >> 6)] |= ...`. This is a **scatter**: writing to an array index that's only known at runtime. NEON has no scatter instruction, so that step stays sequential no matter what.

What I'd actually done was vectorize the cheap arithmetic while leaving the real bottleneck completely untouched. The NEON setup overhead (loading initial offsets, advancing by 4 per round) cost more than the arithmetic I'd saved.

**Result: 5–63% regression across every scenario.**

> **The lesson.** "This loop looks expensive" is not the same as "this loop is the bottleneck." The scatter step was 3 lines of code and it dominated everything. Vectorizing the 3 lines above it accomplished nothing except making the profiler output more confusing. Profile before you optimize. Profile again after. Then profile a third time to be sure.

## Is Rust actually as fast as C++?

The FFmpeg tweet implies Rust is inherently slow. Let's test that directly.

I wrote the exact same blocked filter algorithm in C++: same `probe_masks` scalar loop, same NEON intrinsics, same block layout, compiled at `-O3` with clang. Both the Rust and C++ versions receive the same pre-computed hash values, so neither side has an advantage on hashing.

First, accuracy: the C++ filter produced **bit-for-bit identical false positive rates** to the Rust filter across every scenario. Same algorithm, same results. Good.

Performance, measured in the same benchmark run (compact-128, 4096 queries per batch):

| | Time | ns/op |
|---|---|---|
| `broomfilter-blocked` (Rust) | 45.0 µs | 11.0 ns |
| `cpp-blocked` (C++ via FFI) | 49.8 µs | 12.2 ns |

C++ is **11% slower**. Not 10×. Not 2×. Eleven percent.

And that gap has nothing to do with Rust being intrinsically faster than C++. It's the cost of the FFI boundary. Calling into C++ from Rust means the Rust compiler sees an opaque function it can't look inside. It pays ~1–2 ns per call. Across 4096 queries, that's ~5 µs of pure overhead, which matches the gap almost exactly.

The Rust filter is faster because `hash + probe_masks + neon_contains` are all in the same compilation unit, inlined together into one hot path with no call boundaries in sight. The C++ filter does the same work, just through a door the compiler can't open.

> The Rust code isn't faster because it's Rust. It's faster because there's nothing in the way. Give C++ the same inlining opportunity and the numbers would be identical.

## So, FFmpeg

Back to the tweet. The premise was: move to Rust, get 10× slower, green screens for everyone.

It's a joke. The FFmpeg team knows exactly what they're doing. They write some of the most carefully optimized C and Assembly in existence. The joke works *because* they know what it takes to write fast code, and they know most people don't put in that work.

And here's the thing: they're not wrong about *unoptimized Rust*. A naïve bloom filter in Rust (random cache misses, sequential bit checks, integer division) would be meaningfully slower than the equivalent hand-tuned C. It wouldn't be 10× slower, but it would lose.

The difference between "Rust is slow" and "Rust is fast" is the same as the difference between "C is slow" and "C is fast." It's not the language. It's whether you understand what your hardware is actually doing: where the cache misses are, where the division instructions are, where the sequential bottlenecks are. The FFmpeg team's C is fast because the people writing it know these things cold. Rust can be just as fast for exactly the same reason.

At the end of all this, `broomfilter` sits at **178 million `contains` operations per second** on Apple M4, within 11% of a C++ implementation of the identical algorithm, with the remaining gap entirely explained by a function call.
