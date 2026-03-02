//! Interface for mem structures, including a simulated DRAM bank, CPU registers, paging mechanisms, ...

pub mod addr;
pub mod config;
pub mod paging;

use crate::{
    fault::{Fault, FaultType},
    mem::{config::MEM_CTXT, paging::Page},
};
pub const PAGE_NUMBER: u32 = MEM_CTXT.page_count;
pub const PHYS_TOTAL: usize = (MEM_CTXT.page_count * MEM_CTXT.page_size as u32) as usize;
use config::bitmode::Addr;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

use crate::lang::Byte;

#[derive(Debug, PartialEq, Eq)]
pub enum Segment {
    Stack,
    Neutral,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum BitMode {
    Bit8,
    Bit16,
    Bit32,
    Bit64,
}

type EmptyO = Option<()>;
type MemResult<T> = Result<T, Fault>;

pub struct Stack {
    base: Addr,
    sz: Addr, // Current stack size (i.e., how much of the stack is currently used)
    sp: Addr,
    cap: Addr, // Stack capacity (i.e., maximum stack size, which is the same as the stack segment size)
}

impl Stack {
    // Stack grows upwards, i.e.: `sp` increases when pushing
    // End of written stack (= stack pointer)
    pub fn _end(&self) -> Addr {
        // Exclusive
        self.sp
    }

    // Equivalent to `_end`
    pub fn _sp(&self) -> Addr {
        self.sp
    }

    // End of allocated stack >= `_end`
    pub fn _end_cap(&self) -> Addr {
        self.base + self.cap
    }

    pub fn _push_sp(&mut self) {
        self.sp += 1;
    }

    /// Is `Fault if next operation after this stack pointer push is out of stack bounds
    pub fn _push_sp_checked(&mut self) -> MemResult<()> {
        if self.sp >= self.cap {
            return Err(Fault::_from(FaultType::StackOverflow(0)));
        }
        self.sp += 1;
        Ok(())
    }

    /// Default is bound-zero-checked
    pub fn _pop_sp(&mut self) -> MemResult<()> {
        if self.sp <= 0 {
            return Err(Fault::_from(FaultType::Unrecoverable));
        }
        self.sp -= 1;
        Ok(())
    }
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "┌────────────────────────── Stack ──────────────────────────┐"
        )?;
        writeln!(f, "│ Base:           0x{:08x}", self.base)?;
        writeln!(f, "│ Size:           0x{:08x} ({} bytes)", self.sz, self.sz)?;
        writeln!(
            f,
            "│ Capacity:       0x{:08x} ({} bytes)",
            self.cap, self.cap
        )?;
        writeln!(f, "│ Stack Pointer:  0x{:08x}", self.sp)?;
        writeln!(
            f,
            "│ Used:     {} / {} bytes",
            self.sp - self.base,
            self.cap
        )?;

        let usage_percent = if self.cap > 0 {
            ((self.sp - self.base) as f64 / self.cap as f64) * 100.0
        } else {
            0.0
        };
        writeln!(f, "│ Usage:    {:.1}%", usage_percent)?;
        writeln!(
            f,
            "└───────────────────────────────────────────────────────────┘"
        )?;
        Ok(())
    }
}

impl fmt::Debug for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stack")
            .field("base", &format!("0x{:08x}", self.base))
            .field("size", &format!("0x{:08x}", self.sz))
            .field("capacity", &format!("0x{:08x}", self.cap))
            .field("pointer", &format!("0x{:08x}", self.sp))
            .finish()
    }
}

/// *Physical memory* consisting of one singular bank of SRAM.
///
/// Internally, the bank is made of a capacity-cap-ped `Vec` (of capacity **2^`_PHYS_BITW`**),
/// zinitialised according to (pre-)defined memory context settings, the stack and heap positions within main memory, etc...
#[derive(Debug)]
pub struct Ram {
    _in: Vec<Option<Page>>, // Physical frames; each frame is taken whenever a process allocates it for itself
    pub stack: Stack,
} // (Main, Stack, SP)

impl Ram {
    pub fn new(cap: usize, stack: Stack) -> Ram {
        Ram {
            _in: Vec::with_capacity(cap),
            stack,
        }
    }

    /// The page base address which contains the desired address
    pub fn get_page_of_addr(&self, base: Addr) -> Option<&Page> {
        for p in self._in.iter() {
            if let Some(p) = p {
                if p.ppn_as_addr() == base {
                    return Some(&p);
                }
            }
        }
        None
    }

    pub fn get_page_number_of_addr(&self, base: Addr) -> Option<u32> {
        Some(self.get_page_of_addr(base)?.ppn())
    }
}

impl fmt::Display for Ram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(
            f,
            "╔═══════════════════════════════════════════════════════════╗"
        )?;
        writeln!(
            f,
            "║         RAM (Memory Bank)                                 ║"
        )?;
        writeln!(
            f,
            "╠═══════════════════════════════════════════════════════════╣"
        )?;
        writeln!(f, "{}", self.stack)?;
        writeln!(
            f,
            "╚═══════════════════════════════════════════════════════════╝"
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct MMU {
    pub free_list: Vec<Page>, // Physical frames; each frame is taken whenever a process allocates it for itself
    pub used_list: Vec<Page>,
    pub allocations: HashMap<Addr, usize>, // Keep track of allocated ram blobs (with size) for dealloc and access/info
}

impl MMU {
    pub fn new_init() -> Self {
        let mut free_list = vec![];
        for ppn in 1..MEM_CTXT.page_count {
            free_list.push(Page::new(ppn))
        }
        MMU {
            free_list,
            used_list: vec![],
            allocations: HashMap::new(),
        }
    }

    fn pop_free(&mut self) -> Option<Page> {
        self.free_list.pop().map(|mut page| {
            page.zero();
            page
        })
    }

    fn push_free(&mut self, page: Page) {
        self.free_list.push(page);
    }

    fn push_used(&mut self, page: Page) {
        self.used_list.push(page);
    }

    fn remove_used(&mut self, page: &Page) -> Option<Page> {
        let mut idx = self.used_list.len(); // Any invalid index
        for i in 0..self.used_list.len() {
            if self.used_list[i].ppn == page.ppn {
                idx = i;
            }
        }
        if idx != self.used_list.len() {
            // If the page was actually in there
            return Some(self.used_list.remove(idx));
        }
        return None;
    }

    pub fn free_bytes_amt(&self) -> usize {
        self.free_list
            .iter()
            .map(|_| MEM_CTXT.page_size)
            .into_iter()
            .sum::<usize>()
    }
}

impl fmt::Display for MMU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(
            f,
            "┌───────────── MMU (Memory Management Unit) ──────────────┐"
        )?;

        let free_bytes = self.free_bytes_amt();
        let used_bytes = self.used_list.len() * MEM_CTXT.page_size;
        let percent_used = if PHYS_TOTAL > 0 {
            (used_bytes as f64 / PHYS_TOTAL as f64) * 100.0
        } else {
            0.0
        };
        let percent_free = if PHYS_TOTAL > 0 {
            (free_bytes as f64 / PHYS_TOTAL as f64) * 100.0
        } else {
            0.0
        };

        writeln!(
            f,
            "│ Free Pages:      {} / {}",
            self.free_list.len(),
            self.free_list.len() + self.used_list.len()
        )?;
        writeln!(
            f,
            "│ Used Pages:      {} / {}",
            self.used_list.len(),
            self.free_list.len() + self.used_list.len()
        )?;
        writeln!(
            f,
            "├─────────────────────────────────────────────────────────┤"
        )?;
        writeln!(
            f,
            "│ Free Memory:     {:>8} bytes ({:>5.1}%)",
            free_bytes, percent_free
        )?;
        writeln!(
            f,
            "│ Used Memory:     {:>8} bytes ({:>5.1}%)",
            used_bytes, percent_used
        )?;
        writeln!(f, "│ Total Memory:    {:>8} bytes", PHYS_TOTAL)?;
        writeln!(
            f,
            "├─────────────────────────────────────────────────────────┤"
        )?;
        writeln!(f, "│ Active Allocations: {}", self.allocations.len())?;

        if !self.allocations.is_empty() {
            writeln!(f, "│")?;
            writeln!(f, "│ Allocation Details:")?;
            for (addr, size) in self.allocations.iter() {
                writeln!(f, "│   ├─ 0x{:08x}: {} bytes", addr, size)?;
            }
        }

        writeln!(
            f,
            "└─────────────────────────────────────────────────────────┘"
        )?;
        Ok(())
    }
}

/// A SRAM bank model, acting as a mere allocation pool allied with memory context and an [MMU][MMU].
#[derive(Debug)]
pub struct Memory {
    pub mmu: MMU,
    pub ram: Ram,
}

impl Memory {
    pub fn new() -> Self {
        let stack = Stack {
            base: MEM_CTXT.stack_base as Addr,
            sz: 0,
            cap: MEM_CTXT.stack_sz as Addr,
            sp: MEM_CTXT.stack_base,
        };

        Self {
            mmu: MMU::new_init(),
            ram: Ram::new(MEM_CTXT.page_count as usize, stack), // Allocate for `page_count` pages
        }
    }

    /// Get the segment type of word at `addr`
    pub fn get_segment_type_of(&self, addr: Addr) -> Segment {
        if (self.ram.stack.base..self.ram.stack._end_cap()).contains(&addr) {
            Segment::Stack
        } else {
            Segment::Neutral
        }
    }

    /// Get the page of word at `addr`
    pub fn get_page_of(&self, addr: Addr) -> Option<&Page> {
        self.ram.get_page_of_addr(addr)
    }

    // All access operations are physical-level //
    // and thus should be invoked after translation process //
    // i.e.: `addr` is a physical address //

    fn read_at<T>(&self, addr: Addr) -> MemResult<Vec<Byte>> {
        Ok(self
            .ram
            ._in
            .get((addr / (MEM_CTXT.page_size as u32)) as usize)
            .ok_or(Fault::_from(FaultType::AddrOutOfRange(addr)))?
            .ok_or(Fault::_from(FaultType::InvalidPage))?
            .read::<T>(addr)
            .to_vec())
    }

    /// Reads word at `addr`, checking if this word is allocated yet.
    pub fn _read_at_checked<T>(&self, addr: Addr) -> Option<MemResult<Vec<u8>>> {
        self.mmu.allocations.get(&addr)?;

        Some(self.read_at::<T>(addr))
    }

    /// Writes bytes at `addr`
    /// Mostly used in a non-allocation-guarded context.
    fn _write_at_addr<T>(&mut self, addr: Addr, bytes: &[u8]) -> EmptyO {
        // e.g.: Write no-alloc
        self.ram
            ._in
            .get_mut((addr / (MEM_CTXT.page_size as u32)) as usize)
            .ok_or(Fault::_from(FaultType::AddrOutOfRange(addr)))
            .ok()?
            .as_mut()?
            .write::<T>(addr, bytes);
        Some(())
    }

    /// Writes a singular byte at `addr`, checking if this word is allocated yet.
    /// To be used in an allocation-guarded context.
    pub fn _write_at_addr_checked<T>(&mut self, addr: Addr, bytes: &[Byte]) -> EmptyO {
        self.mmu.allocations.get(&addr).is_none().then(|| ())?;

        self._write_at_addr::<T>(addr, bytes);
        Some(())
    }

    /// Allocates (without writing) `n` consecutive bytes starting at address `addr`.
    /// /!\ Low-level alloc != translation process
    pub fn _alloc(&mut self, addr: Addr, n: usize) {
        self.mmu.allocations.insert(addr, n);
        // let page_at_addr = self.get_page_of(addr); // Physical frame containing that address
    }

    // Checks for no conflict with stack and other already present allocations
    pub fn _alloc_checked(&mut self, addr: Addr, n: usize) {
        if self.mmu.allocations.get(&addr).is_some() {
            () // Don't allocate if there is already an allocation at that address, to prevent conflict with other processes' allocations
        } else if let Segment::Stack = self.get_segment_type_of(addr) {
            () // Don't allocate if the address is in the stack segment, to prevent conflict with stack operations (push/pop)
        } else {
            self._alloc(addr, n);
        }
    }

    /// Deallocates without checking if there is any conflict with the stack or other processes' allocations
    /// (one process can deallocate the resources of another through this method)
    pub fn _dealloc(&mut self, addr: Addr) -> EmptyO {
        self.mmu.allocations.remove(&addr).map(|_| ())
    }

    pub fn _dealloc_check_no_other(&mut self, addr: Addr, id: u8 /* Self id */) -> EmptyO {
        let at_page = self.get_page_of(addr)?;
        match at_page.proc_id {
            Some(proc_id) => {
                if id == proc_id {
                    self._dealloc(addr)
                } else {
                    None
                }
            }
            None => None,
        }
    }

    /// Deallocates only if it's out of the stack (which normally requires the user to call through pop)
    pub fn _dealloc_check_no_stack(&mut self, addr: Addr) -> EmptyO {
        if let Segment::Neutral = self.get_segment_type_of(addr) {
            self.mmu.allocations.remove(&addr).map(|_| ());
            Some(())
        } else {
            None // Warn that nothing could be deallocated
        }
    }

    // Always push a singular byte from stack using `_push` only
    pub fn _push(&mut self, byte: Byte) -> MemResult<()> {
        self._write_at_addr::<Byte>(self.ram.stack.sp, &[byte]);
        self.ram.stack._push_sp();
        Ok(())
    }

    // Always pop a singular byte from stack using `_pop` only
    pub fn _pop(&mut self) -> MemResult<Byte> {
        self.ram.stack._pop_sp()?; // Pop occurs before to prevent reference conflict
        let r = self.read_at::<Byte>(self.ram.stack.sp)?;
        Ok(r[0])
    }
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(
            f,
            "╔═══════════════════════════════════════════════════════════╗"
        )?;
        writeln!(
            f,
            "║             COMPLETE MEMORY STATE REPORT                  ║"
        )?;
        writeln!(
            f,
            "╚═══════════════════════════════════════════════════════════╝"
        )?;
        writeln!(f, "{}", self.ram)?;
        writeln!(f)?;
        writeln!(f, "{}", self.mmu)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod tests {
        use super::super::*;

        #[test]
        fn test_stack_push_sp() {
            let mut stack = Stack {
                base: 0,
                sz: 100,
                sp: 0,
                cap: 100,
            };
            stack._push_sp();
            assert_eq!(stack._sp(), 1);
        }

        #[test]
        fn test_stack_pop_sp() {
            let mut stack = Stack {
                base: 0,
                sz: 100,
                sp: 10,
                cap: 100,
            };
            assert!(stack._pop_sp().is_ok());
            assert_eq!(stack._sp(), 9);
        }

        #[test]
        fn test_stack_pop_sp_underflow() {
            let mut stack = Stack {
                base: 0,
                sz: 100,
                sp: 0,
                cap: 100,
            };
            assert!(stack._pop_sp().is_err());
        }

        #[test]
        fn test_stack_push_sp_checked_overflow() {
            let mut stack = Stack {
                base: 0,
                sz: 100,
                sp: 100,
                cap: 100,
            };
            assert!(stack._push_sp_checked().is_err());
        }

        #[test]
        fn test_memory_get_segment_stack() {
            let memory = Memory::new();
            let addr = memory.ram.stack.base;
            assert_eq!(memory.get_segment_type_of(addr), Segment::Stack);
        }

        #[test]
        fn test_memory_get_segment_neutral() {
            let memory = Memory::new();
            let addr = memory.ram.stack._end_cap() + 100;
            assert_eq!(memory.get_segment_type_of(addr), Segment::Neutral);
        }

        #[test]
        fn test_mmu_new_init() {
            let mmu = MMU::new_init();
            assert!(!mmu.free_list.is_empty());
            assert!(mmu.used_list.is_empty());
        }

        #[test]
        fn test_memory_alloc_and_dealloc() {
            let mut memory = Memory::new();
            memory._alloc(1000, 64);
            assert!(memory.mmu.allocations.contains_key(&1000));
            assert!(memory._dealloc(1000).is_some());
            assert!(!memory.mmu.allocations.contains_key(&1000));
        }

        #[test]
        fn test_memory_alloc_checked_conflict() {
            let mut memory = Memory::new();
            memory._alloc(1000, 64);
            memory._alloc_checked(1000, 65); // Should not allocate again
            assert_eq!(memory.mmu.allocations.get(&1000), Some(&64));
        }

        #[test]
        fn test_memory_alloc_checked_conflit_positive() {
            let mut memory = Memory::new();
            memory._alloc(1000, 64);
            memory._alloc(1000, 65); // Should allocate again
            assert_eq!(memory.mmu.allocations.get(&1000), Some(&65));
        }

        #[test]
        #[should_panic]
        fn test_memory_alloc_and_dealloc_double_free() {
            let mut memory = Memory::new();
            memory._alloc(1000, 64);
            assert!(memory.mmu.allocations.contains_key(&1000));
            assert!(memory._dealloc(1000).is_some());
            assert!(memory._dealloc(1000).is_some());
            // assert!(!memory.mmu.allocations.contains_key(&1000));
        }

        #[test]
        fn test_pretty_print() {
            let mut mem = Memory::new();
            mem._alloc(1000, 64);
            mem._alloc(1010, 64);
            println!("{}", mem);
        }
    }
}
