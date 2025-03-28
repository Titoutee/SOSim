# SOSim (StackOverflow Sim)

#### SOS, I needed simulated paging-based allocation machinery for playing with different buffer-overflow attack patterns in my "TIPE"'s context!

This repo is officially supposed to be an extraction from the paging-based mem-alloc mechanisms taken out of my recent operating system project [BurritOS](https://github.com/Titoutee/BurritOS), which was entirely designed following [Oppermann's Article on Writing an OS in Rust](https://os.phil-opp.com/). It exists for the sake of next year's competitive oral exam, for which I chose to opt for the presentation, demonstration and solving of buffer-overflow-related issues. Thus, beyond its stupid name, **it will indeed not focus strictly on s-o attacks, but on more general-class b-o exploits**.

Despite obvious redundancy, I decided to make this piece of mechanism standalone given the strenuousness of giving away implementation details concerning my whole operating system (and given the severe rules concerning what pieces of hard/software are legal to be used on the d-day).

*I follow here the premature, and maybe erroneous hypothesis that jury members will accept making the effort to read and understand Rust snippets; otherwise break a leg, I guess, and pretend to fall in love with Rustlang's wild and magnificent concepts.*

## Motivation

The whole point of my simulator is however to keep things simple: I just need a support for demonstrating the witty and egregious impacts of buffer-overflow attacks in a user-friendly way. Given that the jury will strictly see snippets screenshots, **the focus is nor on time and memory efficiency, neither on writing beauty.** (message conveyed to GH bad code stalkers)

*"Why is the repo even public then?"* Well, it may be interesting to give people an idea to set sail for :p

## The Simulator

Following the same pattern as BurritOS, the SOSim micro-kernel will run on a QEMU instance, bootloaded by the `bootimage` bootable disk image creating tool.
