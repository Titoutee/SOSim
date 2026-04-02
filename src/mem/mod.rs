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
use std::sync::{Arc, Mutex};

pub const PAGE_NUMBER: u32 = MEM_CTXT.page_count;
pub const PHYS_TOTAL: usize = (MEM_CTXT.page_count * MEM_CTXT.page_size as u32) as usize;

use config::bitmode::Addr;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
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
pub type MemResult<T> = Result<T, Fault>;

pub struct Stack {
    base: Addr,
    sp: Addr,
    cap: Addr, // Stack capacity (i.e., maximum stack size, which is the same as the stack segment size)
}

lazy_static! {
    pub static ref MEMORY: Arc<Mutex<Memory>> = Arc::new(Mutex::new(Memory::new()));
}

impl Default for Stack {
    fn default() -> Self {
        Stack {
            base: MEM_CTXT.stack_base,
            cap: MEM_CTXT.stack_sz,
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
        Stack { base, cap, sp: 0 }
    }
}

/// *Physical memory* consisting of one singular bank of SRAM.

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

    pub fn get_ppns_for_range(&self, addr: Addr, n: usize) -> Vec<u32> {
        // Using get_ppn_of_base_addr to get all the ppns
        let mut ppns = HashSet::new();
        for offset in 0..n as u32 {
            if let Ok(ppn) = self.get_ppn_of_addr(addr + offset) {
                ppns.insert(ppn);
            }
        }
        ppns.into_iter().collect()
    }

    /// Get the ppn of the page containing the given address, if it exists (i.e., if the address is in a page that is either allocated or free)
    /// This takes into account the used and free list, since the page containing the address can be either allocated or free; if it's allocated, it will be in the used list, and if it's free, it will be in the free list
    pub fn get_ppn_of_addr(&self, addr: Addr) -> MemResult<u32> {
        if addr > PHYS_TOTAL as u32 {
            Err(Fault::_from(FaultType::AddrOutOfRange(addr)))
        } else {
            Ok(MEM_CTXT.zero_page_ppn + (addr / MEM_CTXT.page_size as u32))
        }
    }

    /// Move a single page from free list to used list for the given address
    fn mark_page_as_used(&mut self, addr: Addr) -> MemResult<()> {
        // Get the relevant ppn for the given address
        let ppn = self.get_ppn_of_addr(addr)?;
        println!("{}", ppn);

        // Retrieve the page that this ppn labels
        let page = self.alloc_mut().free_list.remove(&ppn); // Remove the page from the free list
        let page = if let None = page {
            if let None = self.alloc_mut().used_list.get(&ppn) {
                Err(Fault::_from(FaultType::InvalidPage(ppn)))?
            } else {
                self.alloc_mut().used_list.get(&ppn).cloned().unwrap()
            }
        } else {
            page.unwrap()
        };
        // Push the page to the used list = page is allocated!!
        self.alloc_mut().push_used(page);
        Ok(())
    }

    /// Move multiple pages from free_list to used_list for the given address range [addr, addr + size]
    fn mark_pages_as_used(&mut self, addr: Addr, n: usize) -> MemResult<()> {
        let ppns = self.get_ppns_for_range(addr, n);
        for ppn in ppns {
            self.mark_page_as_used(ppn * (MEM_CTXT.page_size as u32))?;
        }
        Ok(())
    }

    /// Move a single page from used_list back to free_list for the given address
    fn mark_page_as_free(&mut self, addr: Addr) -> MemResult<()> {
        // Get the relevant ppn for the given address
        let ppn = self.get_ppn_of_addr(addr)?; // Get the ppn of the page containing the address, to find it in the used list and move it back to the free list

        // Retrieve the page that this ppn labels
        let page = self
            .alloc_mut()
            .used_list
            .remove(&ppn)
            .ok_or(Fault::_from(FaultType::InvalidPage(ppn)))?;
        // Remove the page from the used list
        println!("HURRAYYY");
        // Add the page back to the free list
        self.alloc_mut().push_free(page);
        Ok(())
    }

    /// Move multiple pages used_list to free_list for the given address range [addr, addr + size]
    fn mark_pages_as_free(&mut self, addr: Addr, n: usize) -> MemResult<()> {
        // Get relevant ppns for the given address range
        let ppns = self.get_ppns_for_range(addr, n);
        for ppn in ppns {
            self.mark_page_as_free(ppn * (MEM_CTXT.page_size as u32))?;
        }
        Ok(())
    }

    // Free bytes = free pages, without accounting for inner free bytes
    pub fn free_bytes(&self) -> usize {
        self.alloc.free_list.len() * MEM_CTXT.page_size
    }

    pub fn free_bytes_fine(&self) -> usize {
        todo!()
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

    /// Get the page of word at `addr` in free pages list
    pub fn get_page_of(&self, addr: Addr) -> MemResult<&Page> {
        let ppn = self.get_ppn_of_addr(addr)?;
        self.alloc
            .used_list
            .get(&ppn)
            .ok_or(Fault::_from(FaultType::InvalidPage(addr)))
    }

    pub fn get_page_of_mut(&mut self, addr: Addr) -> MemResult<&mut Page> {
        let ppn = self.get_ppn_of_addr(addr)?;
        self.alloc
            .used_list
            .get_mut(&ppn)
            .ok_or(Fault::_from(FaultType::InvalidPage(addr)))
    }

    // Read
    pub fn _read_at(&self, addr: Addr, len: usize) -> MemResult<Vec<Byte>> {
        self.get_page_of(addr)?.read(addr, len)
    }

    /// Reads word at `addr`, checking if this word is allocated yet.
    pub fn _read_at_checked(&self, addr: Addr, len: usize) -> MemResult<Vec<u8>> {
        let ppn = self.get_ppn_of_addr(addr)?;
        self.alloc_var
            .get(&ppn)
            .ok_or(Fault::_from(FaultType::UnknownVar(addr)))?;
        self._read_at(addr, len)
    }

    /// Writes bytes at `addr`
    /// Mostly used in a non-allocation-guarded context.
    pub fn _write_at_addr(&mut self, addr: Addr, bytes: Vec<Byte>) -> MemResult<()> {
        // e.g.: Write no-alloc
        let page = self.get_page_of_mut(addr)?;

        page.write(addr, bytes)
    }

    /// Writes a singular byte at `addr`, checking if this word is allocated yet.
    /// To be used in an allocation-guarded context.
    pub fn _write_at_addr_checked(&mut self, addr: Addr, bytes: Vec<Byte>) -> MemResult<()> {
        let ppn = self.get_ppn_of_addr(addr)?;
        self.alloc_var
            .get(&ppn)
            .ok_or(Fault::_from(FaultType::UnknownVar(addr)))?;
        self._write_at_addr(addr, bytes)
    }

    /// Sets read/write permissions on pages in the given address range
    fn _set_page_permissions(&mut self, addr: Addr, size: usize, readable: bool, writable: bool) {
        let ppns = self.get_ppns_for_range(addr, size);
        for ppn in ppns {
            if let Some(page) = self.alloc_mut().used_list.get_mut(&ppn) {
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

    /// Allocates (without writing) `n` consecutive bytes starting at address `addr`.
    /// /!\ Low-level alloc != translation process
    pub fn _alloc(&mut self, addr: Addr, n: usize) -> MemResult<()> {
        self.alloc_var.insert(addr, n);
        // If page is not allocated yet, mark it as allocated; if it's already allocated, it means it's already marked as allocated and we just need to set the permissions on it (e.g., in the case of an allocation that overlaps with an existing one, which is allowed in the non-checked version of alloc)
        if self.get_page_of(addr).is_err() {
            self.mark_pages_as_used(addr, n)?;
            // Set read and write permissions on allocated pages
            self._set_page_permissions(addr, n, true, true); // Todo: make this configurable
        }
        Ok(())
    }

    // Checks for no conflict with stack and other already present allocations
    pub fn _alloc_checked(&mut self, addr: Addr, n: usize, shrink: bool) -> MemResult<()> {
        // TODO: `shrink` option to try to fit in the available space if there is a conflict, instead of just returning None
        // If `shrink` is true, it will try to shrink the allocation size to fit in the available space if there is a conflict, instead of just returning None
        if self.alloc_var.get(&addr).is_some() {
            Err(Fault::_from(FaultType::Occupied(addr))) // Don't allocate if there is already an allocation at that address, to prevent conflict with other processes' allocations
        } else if let Segment::Stack = self.get_segment_type_of(addr) {
            Err(Fault::_from(FaultType::BadSegment)) // Don't allocate if the address is in the stack segment, to prevent conflict with stack operations (push/pop)
        } else if self.get_page_of(addr).is_err() {
            Err(Fault::_from(FaultType::AddrOutOfRange(addr))) // Don't allocate if the address is out of physical memory range, to prevent invalid memory access; out of range is also considered as not allocated, to prevent conflict with other processes' allocations
        } else if self.get_page_of(addr).unwrap().ppn() == MEM_CTXT.zero_page_ppn {
            Err(Fault::_from(FaultType::InvalidPage(addr))) // Don't allocate if the address is in the zero page, to prevent invalid memory access
        } else if {
            let new_range = addr..addr + (n) as u32;
            self.alloc_var.iter().any(|(&a, &size)| {
                let real_range = a..(a + size as u32);
                !(new_range.end <= real_range.start || new_range.start >= real_range.end)
            })
        } {
            Err(Fault::_from(FaultType::Occupied(addr))) // Don't allocate if there is an overlap with another allocation, to prevent conflict with other processes' allocations
        } else {
            self._alloc(addr, n)?; // Allocate if there is no conflict
            Ok(())
        }
    }

    /// Deallocates without checking if there is any conflict with the stack or other processes' allocations
    /// (one process can deallocate the resources of another through this method)
    pub fn _dealloc(&mut self, addr: Addr) -> MemResult<()> {
        if let Some(n) = self.alloc_var.remove(&addr) {
            // Clear permissions before freeing the pages
            self._set_page_permissions(addr, n, false, false);
            self.mark_pages_as_free(addr, n)?;
            Ok(())
        } else {
            Err(Fault::_from(FaultType::UnknownVar(addr)))
        }
    }

    // Deallocates only if there is no conflict with other processes' allocations (i.e., only if the allocation to be deallocated belongs to the process itself, which is the common case)
    pub fn _dealloc_check_no_other(
        &mut self,
        addr: Addr,
        id: u8, /* Self id */
    ) -> MemResult<()> {
        let at_page = self.get_page_of(addr)?;
        match at_page.proc_id {
            Some(proc_id) => {
                if id == proc_id {
                    self._dealloc(addr)
                } else {
                    Err(Fault::_from(FaultType::InvalidPage(addr))) // Don't deallocate if the page containing the address is allocated to another process, to prevent conflict with other processes' allocations
                }
            }
            None => Err(Fault::_from(FaultType::AddrOutOfRange(addr))),
        }
    }

    /// Deallocates only if it's out of the stack (which requires the user to call through pop)
    pub fn _dealloc_check_no_stack(&mut self, addr: Addr) -> MemResult<()> {
        if let Segment::Neutral = self.get_segment_type_of(addr) {
            let n = self
                .alloc_var
                .remove(&addr)
                .ok_or(Fault::_from(FaultType::UnknownVar(addr)))?;
            self.mark_pages_as_free(addr, n)?;
            Ok(())
        } else {
            Err(Fault::_from(FaultType::BadSegment))
        }
    }

    // Always push a singular byte from stack using `_push` only
    pub fn _push(&mut self, byte: Byte) -> MemResult<()> {
        self._write_at_addr(self.ram().stack.sp, vec![byte]);
        self.ram_mut().stack._push_sp(); // Push occurs after to prevent reference conflict with the read of the byte at the current stack pointer address
        Ok(())
    }

    pub fn _push_checked(&mut self, byte: Byte) -> MemResult<()> {
        self._write_at_addr(self.ram().stack.sp, vec![byte]);
        self.ram_mut().stack._push_sp_checked()?;
        Ok(())
    }

    pub fn _pop_checked(&mut self) -> MemResult<Byte> {
        self.ram_mut().stack._pop_sp()?; // Pop occurs before to prevent reference conflict
        Ok(self._read_at(self.ram().stack.sp, 1)?[0]) // Only one byte is popped, so we return the first byte of the read result
    }

    // Always pop a singular byte from stack using `_pop` only
    pub fn _pop(&mut self) -> MemResult<Byte> {
        self.ram_mut().stack._pop_sp()?; // Pop occurs before to prevent reference conflict
        Ok(self._read_at(self.ram().stack.sp, 1)?[0])
    }
}

#[cfg(test)]
mod tests_memory {
    // use num::rational;

    use super::*;
    use std::vec;

    #[test]
    fn test_stack_push_sp() {
        let mut stack = Stack {
            base: 0,
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
            sp: 0,
            cap: 100,
        };
        assert!(stack._pop_sp().is_err());
    }

    #[test]
    fn test_stack_push_sp_checked_overflow() {
        let mut stack = Stack {
            base: 0,
            sp: 100,
            cap: 100,
        };
        assert!(stack._push_sp_checked().is_err());
    }

    #[test]
    fn test_memory_get_segment_stack() {
        let memory = Memory::new();
        let addr = memory.ram().stack.base;
        assert_eq!(memory.get_segment_type_of(addr), Segment::Stack);
    }

    #[test]
    fn test_memory_get_segment_neutral() {
        let memory = Memory::new();
        let addr = memory.ram().stack._end_cap() + 100;
        assert_eq!(memory.get_segment_type_of(addr), Segment::Neutral);
    }

    #[test]
    fn test_mmu_new_init() {
        let mmu = Memory::new();
        assert!(!mmu.alloc.free_list.is_empty());
        assert!(mmu.alloc.used_list.is_empty());
    }

    #[test]
    fn test_memory_alloc_and_dealloc() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64);
        assert_eq!(memory.alloc_var.get(&1000), Some(&64));
        memory._dealloc(1000);
        assert!(!memory.alloc_var.contains_key(&1000));
    }

    #[test]
    fn test_memory_alloc_checked_conflict() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64).unwrap();
        memory._alloc_checked(1000, 65, false); // Should not allocate again
        assert_eq!(memory.alloc_var.get(&1000), Some(&64));
    }

    #[test]
    fn test_memory_alloc_checked_conflict_positive() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64).unwrap();
        memory._alloc(1000, 65).unwrap(); // Should allocate again
        assert_eq!(memory.alloc_var.get(&1000), Some(&65));
    }

    #[test]
    fn test_memory_alloc_checked_stack_conflict() {
        let mut memory = Memory::new();
        let stack_addr = memory.ram().stack.base + 10;
        memory._alloc_checked(stack_addr, 64, false); // Should not allocate
        assert!(!memory.alloc_var.contains_key(&stack_addr));
    }

    #[test]
    fn test_memory_alloc_checked_overlap_conflict1() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64).unwrap();
        memory._alloc_checked(1020, 64, false); // Should not allocate due to overlap
        assert!(!memory.alloc_var.contains_key(&1020));
    }

    #[test]
    fn test_memory_alloc_checked_overlap_conflict2() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64).unwrap();
        memory._alloc_checked(990, 20, false); // Should not allocate due to overlap
        assert!(!memory.alloc_var.contains_key(&990));
    }

    #[test]
    fn test_memory_alloc_checked_overlap_conflict3() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64).unwrap();
        memory._alloc_checked(1020, 2, false); // Should not allocate due to overlap
        assert!(!memory.alloc_var.contains_key(&1020));
    }

    #[test]
    fn test_memory_alloc_checked_overlap_conflict4() {
        let mut memory = Memory::new();
        memory._alloc(10029, 64).unwrap();
        memory._alloc_checked(10030, 2, false); // Should not allocate due to overlap
        assert!(!memory.alloc_var.contains_key(&10030));
    }

    #[test]
    #[should_panic]
    fn test_memory_alloc_and_dealloc_double_free() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64).unwrap();
        assert!(memory.alloc_var.contains_key(&1000));
        assert!(memory._dealloc(1000).is_ok());
        assert!(memory._dealloc(1000).is_ok());
        // assert!(!memory.alloc_var.contains_key(&1000));
    }
    #[test]
    fn test_alloc_checked_with_shrink_no_fit() {
        let mut memory = Memory::new();
        memory._alloc(1000, 64).unwrap();
        memory._alloc_checked(1020, 10, true); // Should shrink to 0 bytes and not allocate
        assert!(!memory.alloc_var.contains_key(&1020));
    }

    #[test]
    fn test_get_ppns_for_range() {
        let memory = Memory::new();
        let addr = 4096; // Start of page 1
        let size = 1; // 2 pages of 4096 bytes
        let ppns = memory.get_ppns_for_range(addr, size);
        assert_eq!(ppns, vec![1]);
    }

    #[test]
    fn test_get_ppn_of_base_addr() {
        let memory = Memory::new();
        let addr = 4096; // Start of page 1
        assert_eq!(memory.get_ppn_of_addr(addr), Ok(1));
    }

    #[test]
    fn test_get_ppn_of_base_addr_2() {
        let memory = Memory::new();
        let addr = 236523; // Start of page 57
        assert_eq!(memory.get_ppn_of_addr(addr), Ok(57));
    }

    #[test]
    fn test_get_ppn_of_base_addr_3() {
        let memory = Memory::new();
        let addr = 8001; // Start of page 1
        assert_eq!(memory.get_ppn_of_addr(addr), Ok(1));
    }

    #[test]
    fn test_mark_page_as_used_and_free() {
        let mut memory = Memory::new();
        let addr = 4096; // Start of page 1
        memory.mark_page_as_used(addr).unwrap();
        assert!(
            memory
                .alloc
                .free_list
                .iter()
                .all(|p| *p.0 != addr / (MEM_CTXT.page_size as u32))
        );
        assert!(
            memory
                .alloc
                .used_list
                .iter()
                .any(|p| *p.0 == addr / (MEM_CTXT.page_size as u32))
        );
        memory.mark_page_as_free(addr).unwrap();
        assert!(
            memory
                .alloc
                .free_list
                .iter()
                .any(|p| *p.0 == addr / (MEM_CTXT.page_size as u32))
        );
        assert!(
            memory
                .alloc
                .used_list
                .iter()
                .all(|p| *p.0 != addr / (MEM_CTXT.page_size as u32))
        );
    }

    #[test]
    fn get_ppn_of_addr_not_allocated_address_out_of_range() {
        let memory = Memory::new();
        let addr = 4096; // Start of page 1
        assert!(memory.get_ppn_of_addr(addr).is_ok());
        let addr_unallocated = 4096 * 1000; // Start of page 1000, which is out of range
        assert!(memory.get_ppn_of_addr(addr_unallocated).is_err());
    }

    #[test]
    fn test_pretty_print() {
        let mut mem = Memory::new();
        mem._alloc(100000, 64).unwrap();
        mem._alloc(101000, 64).unwrap();
        println!("{}", mem);
    }
}
