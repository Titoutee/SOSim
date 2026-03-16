//! Interface for mem structures, including a simulated DRAM bank, CPU registers, paging mechanisms, ...

pub mod addr;
pub mod config;
pub mod display;
pub mod paging;

use crate::{
    fault::{Fault, FaultType},
    mem::{
        config::MEM_CTXT,
        paging::{Flag, FrameAllocator, Page},
    },
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

impl Default for Stack {
    fn default() -> Self {
        Stack {
            base: MEM_CTXT.stack_base,
            cap: MEM_CTXT.stack_sz,
            sz: 0,
            sp: MEM_CTXT.stack_base,
        }
    }
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

    pub fn new(base: Addr, cap: Addr) -> Self {
        Stack {
            base,
            cap,
            sz: 0,
            sp: 0,
        }
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
    pub fn get_page_of_base_addr(&mut self, base: Addr) -> Option<&mut Page> {
        for p in self._in.iter_mut() {
            if let Some(p) = p {
                if p.ppn_as_addr() == base {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn get_ppn_of_base_addr(&mut self, base: Addr) -> Option<u32> {
        Some(self.get_page_of_base_addr(base)?.ppn())
    }

    pub fn get_ppn_of_addr(&mut self, addr: Addr) -> Option<u32> {
        self.get_ppn_of_base_addr(addr / (MEM_CTXT.page_size as u32))
    }
}

impl fmt::Display for Ram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.stack)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Memory {
    pub alloc: FrameAllocator,
    pub _ram: Ram,
    pub alloc_var: HashMap<Addr, usize>, // Keep track of allocated ram blobs (with size) for dealloc and access/info
}

impl Memory {
    pub fn new() -> Self {
        let mut free_list = vec![];
        for ppn in 1..MEM_CTXT.page_count {
            free_list.push(Page::new(ppn))
        }
        Memory {
            _ram: Ram::new(MEM_CTXT.page_count as usize, Stack::default()),
            alloc: FrameAllocator::new(),
            alloc_var: HashMap::new(),
        }
    }

    // Mutable reference to the inner frame allocator
    fn alloc_mut(&mut self) -> &mut FrameAllocator {
        &mut self.alloc
    }

    /// Get the set of physical page numbers that cover the address range [addr, addr + size]
    pub fn get_ppns_for_range(&self, addr: Addr, size: usize) -> Vec<u32> {
        let page_size = MEM_CTXT.page_size as u32;
        let start_ppn = addr / page_size;
        let end_addr = addr + size as u32;
        let end_ppn = (end_addr + page_size - 1) / page_size;
        (start_ppn..end_ppn).collect()
    }

    /// Move a single page from free list to used list for the given address
    fn mark_page_as_used(&mut self, addr: Addr) -> Option<()> {
        let ppn = self.ram_mut().get_ppn_of_addr(addr)?;
        let index = self.alloc.get_index_free(ppn)?; // Find the page in the free list by its ppn
        let page = self.alloc_mut().free_list.remove(index); // Remove the page from the free list
        self.alloc_mut().push_used(page); // Add the page to the used list
        Some(())
    }

    /// Move pages from free_list to used_list for the given address range [addr, addr + size]
    fn mark_pages_as_used(&mut self, addr: Addr, n: usize) -> Option<()> {
        let ppns = self.get_ppns_for_range(addr, n);
        for ppn in ppns {
            self.mark_page_as_used(ppn * (MEM_CTXT.page_size as u32))?;
        }
        Some(())
    }

    /// Move pages from used_list back to free_list for the given address
    fn mark_page_as_free(&mut self, addr: Addr) -> Option<()> {
        let ppn = self.ram_mut().get_ppn_of_addr(addr)?; // Get the ppn of the page containing the address, to find it in the used list and move it back to the free list
        let index = self.alloc.get_index_used(ppn)?; // Find the page in the used list by its ppn   
        let page = self.alloc_mut().used_list.remove(index); // Remove the page from the used list
        self.alloc_mut().push_free(page); // Add the page back to the free list
        Some(())
    }

    fn mark_pages_as_free(&mut self, addr: Addr, n: usize) -> Option<()> {
        let ppns = self.get_ppns_for_range(addr, n);
        for ppn in ppns {
            self.mark_page_as_free(ppn * (MEM_CTXT.page_size as u32))?;
        }
        Some(())
    }

    pub fn free_bytes(&self) -> usize {
        self.alloc.free_list.len() * MEM_CTXT.page_size
    }

    pub fn ram(&self) -> &Ram {
        &self._ram
    }

    pub fn ram_mut(&mut self) -> &mut Ram {
        &mut self._ram
    }

    /// Get the segment type of word at `addr`
    pub fn get_segment_type_of(&self, addr: Addr) -> Segment {
        if (self.ram().stack.base..self.ram().stack._end_cap()).contains(&addr) {
            Segment::Stack
        } else {
            Segment::Neutral
        }
    }

    /// Get the page of word at `addr`
    pub fn get_page_of(&mut self, addr: Addr) -> Option<&mut Page> {
        self.ram_mut().get_page_of_base_addr(addr)
    }

    // All access operations are physical-level //
    // and thus should be invoked after translation process //
    // i.e.: `addr` is a physical address //

    fn read_at<T>(&self, addr: Addr) -> MemResult<Vec<Byte>> {
        let page = self
            .ram()
            ._in
            .get((addr / (MEM_CTXT.page_size as u32)) as usize)
            .ok_or(Fault::_from(FaultType::AddrOutOfRange(addr)))?
            .ok_or(Fault::_from(FaultType::InvalidPage))?;

        Ok(page.read::<T>(addr))
    }

    /// Reads word at `addr`, checking if this word is allocated yet.
    pub fn _read_at_checked<T>(&self, addr: Addr) -> Option<MemResult<Vec<u8>>> {
        self.alloc_var.get(&addr)?;
        Some(self.read_at::<T>(addr))
    }

    /// Writes bytes at `addr`
    /// Mostly used in a non-allocation-guarded context.
    fn _write_at_addr<T>(&mut self, addr: Addr, bytes: &[u8]) -> EmptyO {
        // e.g.: Write no-alloc
        let page = self
            .ram_mut()
            ._in
            .get_mut((addr / (MEM_CTXT.page_size as u32)) as usize)
            .ok_or(Fault::_from(FaultType::AddrOutOfRange(addr)))
            .ok()?
            .as_mut()?;

        page.write::<T>(addr, bytes);
        Some(())
    }

    /// Writes a singular byte at `addr`, checking if this word is allocated yet.
    /// To be used in an allocation-guarded context.
    pub fn _write_at_addr_checked<T>(&mut self, addr: Addr, bytes: &[Byte]) -> EmptyO {
        self.alloc_var.get(&addr)?;
        self._write_at_addr::<T>(addr, bytes);
        Some(())
    }

    /// Allocates (without writing) `n` consecutive bytes starting at address `addr`.
    /// /!\ Low-level alloc != translation process
    pub fn _alloc(&mut self, addr: Addr, n: usize) {
        self.alloc_var.insert(addr, n);
        self.mark_pages_as_used(addr, n);
        // Set read and write permissions on allocated pages
        self._set_page_permissions(addr, n, true, true);
    }

    /// Sets read/write permissions on pages in the given address range
    fn _set_page_permissions(&mut self, addr: Addr, size: usize, readable: bool, writable: bool) {
        let ppns = self.get_ppns_for_range(addr, size);
        for ppn in ppns {
            if let Some(page) = self
                .ram_mut()
                ._in
                .get_mut((ppn / (MEM_CTXT.page_size as u32)) as usize)
                .and_then(|p| p.as_mut())
            {
                let mut pte = page.get_pte();
                if readable {
                    pte.set_flag(Flag::Read);
                } else {
                    pte.clear_flag(Flag::Read);
                }
                if writable {
                    pte.set_flag(Flag::Writable);
                } else {
                    pte.clear_flag(Flag::Writable);
                }
                page.set_pte(pte);
            }
        }
    }

    // Checks for no conflict with stack and other already present allocations
    pub fn _alloc_checked(&mut self, addr: Addr, n: usize, shrink: bool) -> EmptyO {
        // TODO: `shrink` option to try to fit in the available space if there is a conflict, instead of just returning None
        // If `shrink` is true, it will try to shrink the allocation size to fit in the available space if there is a conflict, instead of just returning None
        if self.alloc_var.get(&addr).is_some() {
            None // Don't allocate if there is already an allocation at that address, to prevent conflict with other processes' allocations
        } else if let Segment::Stack = self.get_segment_type_of(addr) {
            None // Don't allocate if the address is in the stack segment, to prevent conflict with stack operations (push/pop)
        } else if self.get_page_of(addr).is_none() {
            None // Don't allocate if the address is out of physical memory range, to prevent invalid memory access; out of range is also considered as not allocated, to prevent conflict with other processes' allocations
        } else if self.get_page_of(addr).unwrap().ppn() == MEM_CTXT.zero_page_ppn {
            None // Don't allocate if the address is in the zero page, to prevent invalid memory access
        } else if {
            let new_range = addr..addr + (n) as u32;
            self.alloc_var.iter().any(|(&a, &size)| {
                let real_range = a..(a + size as u32);
                !(new_range.end <= real_range.start || new_range.start >= real_range.end)
            })
        } {
            None // Don't allocate if there is an overlap with another allocation, to prevent conflict with other processes' allocations
        } else {
            self._alloc(addr, n); // Allocate if there is no conflict
            Some(())
        }
    }

    /// Deallocates without checking if there is any conflict with the stack or other processes' allocations
    /// (one process can deallocate the resources of another through this method)
    pub fn _dealloc(&mut self, addr: Addr) -> EmptyO {
        if let Some(n) = self.alloc_var.remove(&addr) {
            // Clear permissions before freeing the pages
            self._set_page_permissions(addr, n, false, false);
            self.mark_pages_as_free(addr, n);
            Some(())
        } else {
            None
        }
    }

    // Deallocates only if there is no conflict with the stack or other processes' allocations (i.e., only if the allocation to be deallocated belongs to the process itself, which is the common case)
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

    /// Deallocates only if it's out of the stack (which requires the user to call through pop)
    pub fn _dealloc_check_no_stack(&mut self, addr: Addr) -> EmptyO {
        if let Segment::Neutral = self.get_segment_type_of(addr) {
            if let Some(n) = self.alloc_var.remove(&addr) {
                self.mark_pages_as_free(addr, n);
                Some(())
            } else {
                None
            }
        } else {
            None // Warn that nothing could be deallocated
        }
    }

    // Always push a singular byte from stack using `_push` only
    pub fn _push(&mut self, byte: Byte) -> MemResult<()> {
        self._write_at_addr::<Byte>(self.ram().stack.sp, &[byte]);
        self.ram_mut().stack._push_sp(); // Push occurs after to prevent reference conflict with the read of the byte at the current stack pointer address
        Ok(())
    }

    pub fn _push_checked(&mut self, byte: Byte) -> MemResult<()> {
        self._write_at_addr::<Byte>(self.ram().stack.sp, &[byte]);
        self.ram_mut().stack._push_sp_checked()?;
        Ok(())
    }

    pub fn _pop_checked(&mut self) -> MemResult<Byte> {
        self.ram_mut().stack._pop_sp()?; // Pop occurs before to prevent reference conflict
        let r = self.read_at::<Byte>(self.ram().stack.sp)?;
        Ok(r[0]) // Only one byte is popped, so we return the first byte of the read result
    }

    // Always pop a singular byte from stack using `_pop` only
    pub fn _pop(&mut self) -> MemResult<Byte> {
        self.ram_mut().stack._pop_sp()?; // Pop occurs before to prevent reference conflict
        let r = self.read_at::<Byte>(self.ram().stack.sp)?;
        Ok(r[0])
    }
}

#[cfg(test)]
mod tests_memory {
    use super::*;

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

    fn test_stack_pop_sp_underflow() {
        let mut stack = Stack {
            base: 0,
            sz: 100,
            sp: 0,
            cap: 100,
        };
        assert!(stack._pop_sp().is_err());
    }

    fn test_stack_push_sp_checked_overflow() {
        let mut stack = Stack {
            base: 0,
            sz: 100,
            sp: 100,
            cap: 100,
        };
        assert!(stack._push_sp_checked().is_err());
    }

    fn test_memory_get_segment_stack() {
        let memory = Memory::new();
        let addr = memory.ram().stack.base;
        assert_eq!(memory.get_segment_type_of(addr), Segment::Stack);
    }

    fn test_memory_get_segment_neutral() {
        let memory = Memory::new();
        let addr = memory.ram().stack._end_cap() + 100;
        assert_eq!(memory.get_segment_type_of(addr), Segment::Neutral);
    }

    fn test_mmu_new_init() {
        let mut mmu = Memory::new();
        assert!(!mmu.alloc.free_list.is_empty());
        assert!(mmu.alloc.used_list.is_empty());
    }

    fn test_memory_alloc_and_dealloc() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64);
        assert!(memory.alloc_var.contains_key(&1000));
        assert!(memory._dealloc(1000).is_some());
        assert!(!memory.alloc_var.contains_key(&1000));
    }

    fn test_memory_alloc_checked_conflict() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64);
        memory._alloc_checked(1000, 65, false); // Should not allocate again
        assert_eq!(memory.alloc_var.get(&1000), Some(&64));
    }

    fn test_memory_alloc_checked_conflit_positive() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64);
        memory._alloc(1000, 65); // Should allocate again
        assert_eq!(memory.alloc_var.get(&1000), Some(&65));
    }

    fn test_memory_alloc_checked_stack_conflict() {
        let mut memory = Memory::new();
        let stack_addr = memory.ram().stack.base + 10;
        memory._alloc_checked(stack_addr, 64, false); // Should not allocate
        assert!(!memory.alloc_var.contains_key(&stack_addr));
    }

    fn test_memory_alloc_checked_overlap_conflict1() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64);
        memory._alloc_checked(1020, 64, false); // Should not allocate due to overlap
        assert!(!memory.alloc_var.contains_key(&1020));
    }

    fn test_memory_alloc_checked_overlap_conflict2() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64);
        memory._alloc_checked(990, 20, false); // Should not allocate due to overlap
        assert!(!memory.alloc_var.contains_key(&990));
    }

    fn test_memory_alloc_checked_overlap_conflict3() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64);
        memory._alloc_checked(1020, 2, false); // Should not allocate due to overlap
        assert!(!memory.alloc_var.contains_key(&1020));
    }

    #[test]
    #[should_panic]
    fn test_memory_alloc_and_dealloc_double_free() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64);
        assert!(memory.alloc_var.contains_key(&1000));
        assert!(memory._dealloc(1000).is_some());
        assert!(memory._dealloc(1000).is_some());
        // assert!(!memory.alloc_var.contains_key(&1000));
    }

    fn test_alloc_checked_with_shrink_no_fit() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64);
        memory._alloc_checked(1020, 10, true); // Should shrink to 0 bytes and not allocate
        assert!(!memory.alloc_var.contains_key(&1020));
    }

    fn test_pretty_print() {
        let mut mem = Memory::new();
        mem._alloc(100000, 64);
        mem._alloc(101000, 64);
        println!("{}", mem);
    }

    #[test]
    fn run_all_memory_tests() {
        println!("stack_push_sp");
        test_stack_push_sp();
        println!("stack_pop_sp");
        test_stack_pop_sp();
        println!("stack_pop_sp_underflow");
        test_stack_pop_sp_underflow();
        println!("stack_push_sp_checked_overflow");
        test_stack_push_sp_checked_overflow();
        println!("memory_get_segment_stack");
        test_memory_get_segment_stack();
        println!("memory_get_segment_neutral");
        test_memory_get_segment_neutral();
        println!("mmu_new_init");
        test_mmu_new_init();
        println!("memory_alloc_and_dealloc");
        test_memory_alloc_and_dealloc();
        println!("memory_alloc_checked_conflict");
        test_memory_alloc_checked_conflict();
        println!("memory_alloc_checked_conflit_positive");
        test_memory_alloc_checked_conflit_positive();
        println!("memory_alloc_checked_stack_conflict");
        test_memory_alloc_checked_stack_conflict();
        println!("memory_alloc_checked_overlap_conflict1");
        test_memory_alloc_checked_overlap_conflict1();
        println!("memory_alloc_checked_overlap_conflict2");
        test_memory_alloc_checked_overlap_conflict2();
        println!("memory_alloc_checked_overlap_conflict3");
        test_memory_alloc_checked_overlap_conflict3();
        println!("alloc_checked_with_shrink_no_fit");
        test_alloc_checked_with_shrink_no_fit();
        println!("pretty_print");
        test_pretty_print();
    }
}
