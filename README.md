# SOSim (StackOverflow Sim)

### SOS, I needed simulated paging-based allocation machinery for playing with different buffer-overflow attack patterns in my "TIPE"'s context!

This repo is officially supposed to be extracted from the paging-based mem-alloc mechanisms taken out of my recent operating system project [BurritOS](https://github.com/Titoutee/BurritOS), which was entirely designed following [Oppermann's Article on Writing an OS in Rust](https://os.phil-opp.com/), unlike being implemented as a virtual machine rather than a `no_std` micro-kernel. 

It exists for the sake of next year's competitive oral exam, for which I chose to opt for the **presentation, demonstration and solving of buffer-overflow related issues**. Thus, beyond its name, **it will indeed not focus strictly on s-o attacks, but on more general-class b-o exploits**.

*I follow here the premature, and maybe erroneous hypothesis that jury members will accept Rustlang as the basis stone of my demonstration.*

### Motivation

The whole point of my simulator is however to keep things simple: I just need a support for demonstrating the witty and egregious impacts of buffer-overflow attacks in a user-friendly way. Given that the jury will strictly see code snippets, **the focus is neither on time and memory efficiency, nor on writing beauty.**

### The Simulator

The main goal of the virtual machine is to **emulate hardware translation mechanisms between address spaces in a paging environment** to give support for the demonstration of buffer overflows, in parallel to giving a bit of context about mem virtualization.

SOSim is intended to be implemented as a simplist virtual machine exposing mem-alloc mechanisms, rather than a micro-kernel, which would add unnecessary performance and implementation overhead.

### What does SOSim simulate?

**A DRAM bank with an address space size ranging from 8-bit to 64-bit addressing** is simulated, to expose a versatile set of mechanisms, trying to get near real-world architectures (e.g. SOSim can emulate a _64-bit v-address space_ and _4-level page tables_ as in _x86\_64_).

Paging is implemented at hand in a very simplistic way, in the most naive way possible, given the fact that mem-virtualization is not at the core of the presentation (albeit being breifly described for a thorough understanding
of the main concept).

For now, any form of DRAM access optimization pattern (TLBs, ...) detail is put apart.

[More about paging and related mechanisms](https://pages.cs.wisc.edu/~remzi/OSTEP/#book-chapters)

**/!\\**
*The simulator does not include CPU emulation; it only serves as a memory simulator*.

_Implementation details will be further documented_

### Using the Simulator

_Implementation details will be further documented_

### Architectures

SOSim permits to **emulate address spaces of different bitsizes**, in order to
observe memory attacks in deeply different contexts, from a **narrow 8-bit space
to a large, _x86\_64_-like 64b space**. Here are the relevant information about
v-address formatting across these different contexts:

#### Bitmodes

Here are referenced the different paging contexts for each bitmode (_64-bit_ sticks to the _x86\_64_ standard)

|   | PT entries | Page size | PT levels | V-addr index length | V-addr offset length |
|:-:|:----------:|:---------:|:---------:|:-------------------:|:--------------------:|
| **8-bit**      |-|-|-|-|-|
| **16-bit**     |-|-|-|-|-|
| **32-bit**     |-|-|-|-|-|
| **64-bit**   |512|4KB|4|9b|12b|

### Address format specification (virtual)

| Bits |  Name  | Meaning |
|:-----|:------:|:-------:|
| **0**   | Present | The page is already present and active in volatile memory |
| **1**   | Write | Write operations are permitted to this page |
| **2**   | Read | Read operations are permitted to this page |
| **3-63** | Address | The address payload, containing extension bits depending on the bitmode |