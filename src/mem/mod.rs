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

use anyhow::Context;
use config::bitmode::Addr;
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
            if let Some(ppn) = self.get_ppn_of_addr(addr + offset) {
                ppns.insert(ppn);
            }
        }
        ppns.into_iter().collect()
    }

    /// Get the ppn of the page containing the given address, if it exists (i.e., if the address is in a page that is either allocated or free)
    /// This takes into account the used and free list, since the page containing the address can be either allocated or free; if it's allocated, it will be in the used list, and if it's free, it will be in the free list
    pub fn get_ppn_of_addr(&self, addr: Addr) -> Option<u32> {
        // Using the used list to find the ppn of the page containing the address, since the free list only contains free pages and thus cannot be used to find the ppn of an allocated page
        for page in self
            .alloc
            .used_list
            .iter()
            .chain(self.alloc.free_list.iter())
        {
            let page_start_addr = page.0 * (MEM_CTXT.page_size as u32);
            let page_end_addr = page_start_addr + (MEM_CTXT.page_size as u32);
            if (page_start_addr..page_end_addr).contains(&addr) {
                return Some(*page.0);
            }
        }
        None
    }

    /// A variant of `get_ppn_of_addr` that only looks at the used list, to find the ppn of the page containing the address if it is allocated, and return None if it's not allocated (i.e., if it's in the free list or out of range)
    pub fn get_ppn_of_addr_page_used_only(&self, addr: Addr) -> Option<u32> {
        // Using the used list to find the ppn of the page containing the address, since the free list only contains free pages and thus cannot be used to find the ppn of an allocated page
        for page in self.alloc.used_list.iter() {
            let page_start_addr = page.0 * (MEM_CTXT.page_size as u32);
            let page_end_addr = page_start_addr + (MEM_CTXT.page_size as u32);
            if (page_start_addr..page_end_addr).contains(&addr) {
                return Some(*page.0);
            }
        }
        None
    }

    /// Move a single page from free list to used list for the given address
    fn mark_page_as_used(&mut self, addr: Addr) -> anyhow::Result<()> {
        // Get the relevant ppn for the given address
        let ppn = self
            .get_ppn_of_addr(addr)
            .context("get ppn of base addr in mark page as used")?;
        println!("{}", ppn);

        // Retrieve the page that this ppn labels
        let page = self.alloc_mut().free_list.remove(&ppn); // Remove the page from the free list
        let page = if let None = page {
            if let None = self.alloc_mut().used_list.get(&ppn) {
                anyhow::bail!("unknown page access")
            } else {
                self.alloc_mut().used_list.get(&ppn).cloned().unwrap()
            }
        } else {
            page.unwrap()
        };
        // Push the page to the used list = page is allocated!!
        self.alloc_mut().push_used(page);
        anyhow::Ok(())
    }

    /// Move multiple pages from free_list to used_list for the given address range [addr, addr + size]
    fn mark_pages_as_used(&mut self, addr: Addr, n: usize) -> anyhow::Result<()> {
        let ppns = self.get_ppns_for_range(addr, n);
        for ppn in ppns {
            self.mark_page_as_used(ppn * (MEM_CTXT.page_size as u32))
                .context("mark page as used in mark pages as used")?;
        }
        anyhow::Ok(())
    }

    /// Move a single page from used_list back to free_list for the given address
    fn mark_page_as_free(&mut self, addr: Addr) -> anyhow::Result<()> {
        // Get the relevant ppn for the given address
        let ppn = self
            .get_ppn_of_addr(addr)
            .context("get ppn of addr in mark page as free")?; // Get the ppn of the page containing the address, to find it in the used list and move it back to the free list

        // Retrieve the page that this ppn labels
        let page = self
            .alloc_mut()
            .used_list
            .remove(&ppn)
            .context("Remove the ppn from the used list")?; // Remove the page from the used list

        // Add the page back to the free list
        self.alloc_mut().push_free(page);
        anyhow::Ok(())
    }

    /// Move multiple pages used_list to free_list for the given address range [addr, addr + size]
    fn mark_pages_as_free(&mut self, addr: Addr, n: usize) -> anyhow::Result<()> {
        // Get relevant ppns for the given address range
        let ppns = self.get_ppns_for_range(addr, n);
        for ppn in ppns {
            self.mark_page_as_free(ppn * (MEM_CTXT.page_size as u32))
                .context("mark page as free in mark pages as free")?;
        }
        anyhow::Ok(())
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
    pub fn get_page_of(&self, addr: Addr) -> Option<&Page> {
        self.alloc.used_list.get(&addr)
    }

    pub fn get_page_of_mut(&mut self, addr: Addr) -> Option<&mut Page> {
        self.alloc.used_list.get_mut(&addr)
    }

    // Read
    fn read_at<T>(&self, addr: Addr) -> MemResult<Vec<Byte>> {
        self.get_page_of(addr)
            .ok_or(Fault::_from(FaultType::InvalidPage))?
            .read::<T>(addr)
    }

    /// Reads word at `addr`, checking if this word is allocated yet.
    pub fn _read_at_checked<T>(&self, addr: Addr) -> MemResult<Vec<u8>> {
        self.alloc_var
            .get(&addr)
            .ok_or(Fault::_from(FaultType::UnknownVar(addr)))?;
        self.read_at::<T>(addr)
    }

    /// Writes bytes at `addr`
    /// Mostly used in a non-allocation-guarded context.
    fn _write_at_addr<T>(&mut self, addr: Addr, bytes: &[u8]) -> Option<()> {
        // e.g.: Write no-alloc
        let page = self.get_page_of_mut(addr)?;

        page.write::<T>(addr, bytes);
        Some(())
    }

    /// Writes a singular byte at `addr`, checking if this word is allocated yet.
    /// To be used in an allocation-guarded context.
    pub fn _write_at_addr_checked<T>(&mut self, addr: Addr, bytes: &[Byte]) -> Option<()> {
        self.alloc_var.get(&addr)?;
        self._write_at_addr::<T>(addr, bytes);
        Some(())
    }

    /// Allocates (without writing) `n` consecutive bytes starting at address `addr`.
    /// /!\ Low-level alloc != translation process
    pub fn _alloc(&mut self, addr: Addr, n: usize) -> anyhow::Result<()> {
        self.alloc_var.insert(addr, n);
        // If page is not allocated yet, mark it as allocated; if it's already allocated, it means it's already marked as allocated and we just need to set the permissions on it (e.g., in the case of an allocation that overlaps with an existing one, which is allowed in the non-checked version of alloc)
        if self.get_page_of(addr).is_none() {
            self.mark_pages_as_used(addr, n)
                .context("mark page as used in _alloc")?;
            // Set read and write permissions on allocated pages
            self._set_page_permissions(addr, n, true, true);
        }
        Ok(())
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
    pub fn _alloc_checked(&mut self, addr: Addr, n: usize, shrink: bool) -> anyhow::Result<()> {
        // TODO: `shrink` option to try to fit in the available space if there is a conflict, instead of just returning None
        // If `shrink` is true, it will try to shrink the allocation size to fit in the available space if there is a conflict, instead of just returning None
        if self.alloc_var.get(&addr).is_some() {
            anyhow::bail!("Address already allocated"); // Don't allocate if there is already an allocation at that address, to prevent conflict with other processes' allocations
        } else if let Segment::Stack = self.get_segment_type_of(addr) {
            anyhow::bail!("Address in stack segment"); // Don't allocate if the address is in the stack segment, to prevent conflict with stack operations (push/pop)
        } else if self.get_page_of(addr).is_none() {
            anyhow::bail!("Address out of physical memory range"); // Don't allocate if the address is out of physical memory range, to prevent invalid memory access; out of range is also considered as not allocated, to prevent conflict with other processes' allocations
        } else if self.get_page_of(addr).unwrap().ppn() == MEM_CTXT.zero_page_ppn {
            anyhow::bail!("Address in zero page"); // Don't allocate if the address is in the zero page, to prevent invalid memory access
        } else if {
            let new_range = addr..addr + (n) as u32;
            self.alloc_var.iter().any(|(&a, &size)| {
                let real_range = a..(a + size as u32);
                !(new_range.end <= real_range.start || new_range.start >= real_range.end)
            })
        } {
            anyhow::bail!("Address conflicts with existing allocation"); // Don't allocate if there is an overlap with another allocation, to prevent conflict with other processes' allocations
        } else {
            self._alloc(addr, n).context("raw alloc in alloc checked")?; // Allocate if there is no conflict
            Ok(())
        }
    }

    /// Deallocates without checking if there is any conflict with the stack or other processes' allocations
    /// (one process can deallocate the resources of another through this method)
    pub fn _dealloc(&mut self, addr: Addr) -> anyhow::Result<()> {
        if let Some(n) = self.alloc_var.remove(&addr) {
            // Clear permissions before freeing the pages
            self._set_page_permissions(addr, n, false, false);
            self.mark_pages_as_free(addr, n)
                .context("mark pages as free")?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Address not found"))
        }
    }

    // Deallocates only if there is no conflict with the stack or other processes' allocations (i.e., only if the allocation to be deallocated belongs to the process itself, which is the common case)
    pub fn _dealloc_check_no_other(
        &mut self,
        addr: Addr,
        id: u8, /* Self id */
    ) -> anyhow::Result<()> {
        let at_page = self.get_page_of(addr).context("get page")?;
        match at_page.proc_id {
            Some(proc_id) => {
                if id == proc_id {
                    self._dealloc(addr)
                } else {
                    Err(anyhow::anyhow!("Allocation does not belong to the process"))
                }
            }
            None => Err(anyhow::anyhow!("Address not found")),
        }
    }

    /// Deallocates only if it's out of the stack (which requires the user to call through pop)
    pub fn _dealloc_check_no_stack(&mut self, addr: Addr) -> anyhow::Result<()> {
        if let Segment::Neutral = self.get_segment_type_of(addr) {
            if let Some(n) = self.alloc_var.remove(&addr) {
                self.mark_pages_as_free(addr, n)
                    .context("mark pages as free")?;
                Ok(())
            } else {
                Err(anyhow::anyhow!("Address not found"))
            }
        } else {
            Err(anyhow::anyhow!("Address is not in the neutral segment"))
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
        Ok(self.read_at::<Byte>(self.ram().stack.sp)?[0]) // Only one byte is popped, so we return the first byte of the read result
    }

    // Always pop a singular byte from stack using `_pop` only
    pub fn _pop(&mut self) -> MemResult<Byte> {
        self.ram_mut().stack._pop_sp()?; // Pop occurs before to prevent reference conflict
        Ok(self.read_at::<Byte>(self.ram().stack.sp)?[0])
    }
}

#[cfg(test)]
mod tests_memory {
    // use num::rational;

    use std::vec;

    use super::*;

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
        assert_eq!(memory.get_ppn_of_addr(addr), Some(1));
    }

    #[test]
    fn test_get_ppn_of_base_addr_2() {
        let memory = Memory::new();
        let addr = 236523; // Start of page 57
        assert_eq!(memory.get_ppn_of_addr(addr), Some(57));
    }

    #[test]
    fn test_get_ppn_of_base_addr_3() {
        let memory = Memory::new();
        let addr = 8001; // Start of page 1
        assert_eq!(memory.get_ppn_of_addr(addr), Some(1));
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
    fn test_pretty_print() {
        let mut mem = Memory::new();
        mem._alloc(100000, 64).unwrap();
        mem._alloc(101000, 64).unwrap();
        println!("{}", mem);
    }
}
