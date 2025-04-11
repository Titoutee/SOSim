# SOSim (StackOverflow Sim)

#### SOS, I needed simulated paging-based allocation machinery for playing with different buffer-overflow attack patterns in my "TIPE"'s context!

This repo is officially supposed to be an extraction from the paging-based mem-alloc mechanisms taken out of my recent operating system project [BurritOS](https://github.com/Titoutee/BurritOS), which was entirely designed following [Oppermann's Article on Writing an OS in Rust](https://os.phil-opp.com/), unlike being implemented as a virtual machine rather than a `no_std` micro-kernel. It exists for the sake of next year's competitive oral exam, for which I chose to opt for the presentation, demonstration and solving of buffer-overflow-related issues. Thus, beyond its stupid name, **it will indeed not focus strictly on s-o attacks, but on more general-class b-o exploits**.

*I follow here the premature, and maybe erroneous hypothesis that jury members will accept making the effort to read and understand Rust snippets; otherwise break a leg, I guess, and pretend to fall in love with Rustlang's wild and magnificent concepts.*

## Motivation

The whole point of my simulator is however to keep things simple: I just need a support for demonstrating the witty and egregious impacts of buffer-overflow attacks in a user-friendly way. Given that the jury will strictly see snippets screenshots, **the focus is nor on time and memory efficiency, neither on writing beauty.**

## The Simulator

SOSim is intended to be implemented as a simplist virtual machine exposing memalloc mechanisms, rather than a micro-kernel, which would add unnecessary performance and implementation overhead.

The main goal of the VM is to emulate hardware translation mechanisms between address spaces in a paging environment to give support for the demonstration of buffer overflows, in parallel to giving a bit of context about mem virtualization.

## What does SOSim simulate?

A DRAM bank with an address space size ranging from 8-bit to 64-bit addressing is simulated, to expose a versatile set of mechanisms, trying to get near real-world-looking architectures (e.g. SOSim can emulate a 64-bit v-address space and 4-level page tables as in x86_64).

Paging is implemented at hand in a very simplistic way, in the most naive way possible, given the fact that mem-virtualization is not at the core of the presentation.

For now, any form of DRAM access optimization pattern (TLBs, ...) detail is put apart.

[For more about paging](https://pages.cs.wisc.edu/~remzi/OSTEP/#book-chapters)

**/!\\**
*The simulator does not include CPU emulation; it only serves as a memory simulator*.

**Implementation details will be further documented.**

## Using the Simulator

**TODO**

