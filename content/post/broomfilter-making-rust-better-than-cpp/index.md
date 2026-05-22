---
title: 'Broomfilter: making Rust faster than C'
published: 2026-04-12
draft: true
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

- Yes, a few people know 

## Ragebait and FFMPEG

On the sacred day of the 1st of April, I came across [this tweet](https://x.com/ffmpeg/status/2039115531744334180) from the offical FFmepg account. And I have to admit, I was hurt.

Of course, I know how much effort goes into making FFmpeg fast. And I had heard of how Rust could sometimes be suprisingly bad for high performance tasks, but it was nothing concrete.

So, as we all love to do, I wanted to prove these guys wrong. I wanted to show that it was POSSIBLE to make Rust as fast as well optimized C code.




