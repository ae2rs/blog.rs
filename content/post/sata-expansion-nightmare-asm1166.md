---
title: 'PCIe SATA expansion and the ASM1166 nightmare'
published: 2025-11-10
draft: false
---

### TLDR

Check if your SATA PCIe expansion board uses `ASM1166`. Run the following command:

```sh
lspci -nn | grep -i sata
````

Here is an example output from my TrueNAS:

```sh
truenas_admin@truenas[~]$ lspci -nn | grep -i sata
02:00.1 SATA controller [0106]: Advanced Micro Devices, Inc. [AMD] 500 Series Chipset SATA Controller [1022:43eb]
07:00.0 SATA controller [0106]: ASMedia Technology Inc. ASM1166 Serial ATA Controller [1b21:1166] (rev 02)
```

As you can see, the second entry lists `ASMedia Technology Inc. ASM1166 Serial ATA Controller`.

If this is the case for you and you are experiencing compatibility and disconnect issues with ASM1166 on Linux (e.g., TrueNAS, Unraid, etc.), go to [this section](#how-to-fix) and follow the instructions.

## Foreword

This is a pretty short article — bite-sized, I would even say.

It was not planned at all and is not exactly up to my standards of quality, but I want to avoid other hobbyists going through the same pain that I did.

If you find this article useful, don't hesitate to take a look at the cool (chaotic) stuff I do here!

In any case, enjoy your reading!

# Firmwares are the worst

I'm a pretty seasoned self-hoster and over the years I've seen *A LOT* of sh*t software. And by far, firmwares are the most annoying to deal with.

Here are my 4 reasons why firmwares are a PITA to deal with:

1. Documentation is (almost) always non-existent
2. Googling things only gets you to 15-year-old forum discussions which end with “just buy XXX different hardware”
3. Manufacturers seem to compete in “who will bury the firmware updates the deepest”

And the best of the best:

> FIRMWARE UPDATE TOOLS ALWAYS REQUIRE THE ANTICHRIST OF OSs: WINDOWS

No seriously, I hate this. And the cheaper the card you buy, the worse it gets. Some manufacturers don't even bother providing firmware images — let alone update tools.

## A bit of context

Lately, I've been rebuilding my entire homelab from the ground up. My needs (and those of my family) increased, and my janky, budget fire-hazard of a NAS wasn't cutting it anymore.

I needed to *SCALE*.

But since I didn't want to spend 2000€ on a new NAS build but still wanted *relatively* new hardware, I made some choices.

One of them was using a cheap [PCIe SATA expansion card](https://www.amazon.fr/s?k=sata+pcie+x4&crid=7P55K0PQE548&sprefix=SATA+PCIE%2Caps%2C89&ref=nb_sb_ss_mvt-t11-ranker_3_9) from Amazon (yes, `amazon.fr` — I live in France). A similar card had worked flawlessly for me for 1.5 years, so I was pretty confident picking this part, thinking *nothing* would go wrong on that side except maybe reduced [C-States](https://metebalci.com/blog/a-minimum-complete-tutorial-of-cpu-power-management-c-states-and-p-states/) **foreshadowing intensifies**.

All the parts arrived, and I joyfully built my new NAS and installed [TrueNAS](https://www.truenas.com/truenas-community-edition/), which is now based on Debian for both Core and Scale versions. I created my VDEVs and pools, and everything was going fine. The drives showed up, the [SMART](https://fr.wikipedia.org/wiki/Self-Monitoring,_Analysis_and_Reporting_Technology) values were properly being reported, and the performance was good!

But things started to go sideways when I began to look into power consumption.

## Why bother?

See, I live in France, and thanks to some wonderful European laws, I pay 0.23€ per kWh.

It might not seem like much, but some quick math — or using [this incredible tool](https://www.calculator.net/electricity-calculator.html?appliance=&power=180&powerunit=W&capacity=100&usage=24&usageunit=hpd&price=0.23&x=Calculate) — shows that a few appliances running 24/7 and consuming an average of 180W end up costing a whopping 362.91€ a year!

Three years of this and I could buy a second NAS!

So I started to dig into basic power efficiency tuning. And I knew that bad PCIe cards are a typical culprit of high power consumption, being both power-inefficient and blocking deep C-States.

But I was not ready for what happened next.

My first reflex was to use the masterpiece that is `powertop`.

I simply ran:

```sh
truenas_admin@truenas[~]$ sudo powertop --auto-tune
modprobe cpufreq_stats failed
Loaded 0 prior measurements
RAPL device for cpu 0
RAPL Using PowerCap Sysfs : Domain Mask 5
RAPL device for cpu 0
RAPL Using PowerCap Sysfs : Domain Mask 5
Devfreq not enabled
glob returned GLOB_ABORTED
Leaving PowerTOP
```

At first, all was going well. But then my shell froze. I tried to refresh the page — and now my entire NAS was unreachable.

I was panicking. I wasn't home, so I couldn't reboot it by pressing the physical button, and I didn't want to wait to come back from my vacation.

So I insisted. And insisted. And after refreshing the page for 5 long minutes, the login page came back up. My NAS had crashed and rebooted apparently. And of course, all of the powertop changes were gone.

But that was not the worst. Two of my 8 TB disks were showing errors. With my [RAIDZ2](https://www.raidz-calculator.com/raidz-types-reference.aspx) setup, that meant the pool was still usable, but even one more drive failure would render it unusable.

Thankfully, that also gave me a hint that the SATA expansion card was the problem.

The two failing drives were the only two drives from the main pool that were connected through the PCIe SATA card and not directly to the motherboard.

So I tried everything.

First, I just rebooted without changing anything, which made the two erroring drives come back up (yayyy!).

I tried tuning powertop manually, disabling disk spindown, disabling ASPM, but nothing worked. Whenever I tried making adjustments to power consumption, the drives connected to the PCIe SATA card would end up disconnecting.

## How (not) to fix

A bit of googling made it pretty obvious that it was a firmware problem for the well-known `ASM1166` chip (hence the title).

Some retailers sell cards with firmware that is 5+ years old, and it causes a ton of different problems — like incompatibility with Intel 600 or newer motherboards (I’ve got AMD), or disconnection issues when using ASPM (that’s me).

But pretty much every resource I found out there — forum topics, Reddit comments, and even the manufacturer’s website — required me to pull the card out of my NAS and into a *Windows* computer to update the firmware.

Here's the thing: I don't have a single Windows computer. I just don't. My work computer is a Mac, and my main desktop runs Arch.

So I was stuck.

# How to fix

So here is how I actually fixed ALL of my issues.

First, download the flash tool from Radxa:

* [original source](https://dl.radxa.com/accessories/m2-to-hexa-sata-adapter/tools/)
* [my mirror](https://drive.proton.me/urls/E4FA1W7D34#PF5iYvLUUeDc) with password `decastro.dev` (last update: 10-11-2025)

It should include the latest firmware `.ROM` image, but you can also find it on the chip manufacturer's website [here](https://www.station-drivers.com/index.php/th/component/remository/func-startdown/6611/lang,th-th/).

Download it directly using wget:

```sh
wget https://www.station-drivers.com/download/asmedia/asmedia_sata_firmware(station-drivers.com).zip
```

Check that the flashing tool binary works correctly by printing the version number:

```sh
sudo ./116xfwdl -S
```

Now you can decompress the ROM archive (using `unzip`, for example) and run the flashing tool with root privileges:

```sh
sudo ./116xfwdl -U 11080000.ROM
```

You should (hopefully) get a success message.

You can now reboot and enjoy what should have been the case from the beginning: a working PCIe SATA card!

# Conclusion

If you’ve made it this far — congrats, you’ve probably just survived the `ASM1166` nightmare.
It’s ridiculous that a simple SATA controller can cause this much chaos, but hey — that’s the joy of self-hosting, right?

With the right firmware and a bit of persistence, your setup should finally be stable, efficient, and crash-free (hopefully).

Until the next obscure hardware bug shows up, at least.

Hope you enjoyed and found this (somewhat) useful. See you next time! ^^

