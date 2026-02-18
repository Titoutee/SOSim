//! Interface for mem structures, including a simulated DRAM bank, CPU registers, paging mechanisms, ...

pub mod addr;
pub mod config;
pub mod paging;

use crate::{
    fault::{Fault, FaultType},
    mem::{
        config::{_STACK_BASE, _STACK_SZ, MEM_CTXT, MemContext},
        paging::Page,
    },
};
pub const PAGE_NUMBER: u32 = MEM_CTXT.page_count;
pub const PHYS_TOTAL: usize = (MEM_CTXT.page_count * MEM_CTXT.page_size as u32) as usize;
use config::bitmode::Addr;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json;
use std::{collections::HashMap, fs, io::Empty};

use crate::lang::{Byte, Struct};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Stack {
    base: Addr,
    sz: Addr,
    sp: Addr,
    cap: Addr,
}

impl Stack {
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
        // Exclusive
        self.base + self.sz
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

    pub fn dbg(&self) {
        println!("[Stack]");
        println!("  - Stack size: {:x}", self.sz);
        println!("  - Stack capacity: {:x}", self.cap);
        println!("  - Stack base: {:x}", self.base);
        println!("  - Stack pointer: {:x}", self.sp);
    }
}

/// *Physical memory* consisting of one singular bank of SRAM.
///
/// Internally, the bank is made of a capacity-cap-ped `Vec` (of capacity **2^`_PHYS_BITW`**),
/// zinitialised according to (pre-)defined memory context settings, the stack and heap positions within main memory, etc...
#[derive(Debug)]
pub struct Ram {
    pub _in: Vec<Page>,
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
            if p.addr == base {
                return Some(&p);
            }
        }
        None
    }

    pub fn dbg(&self) {
        println!("[RAM]");
        self.stack.dbg();
    }
}

#[derive(Debug)]
pub struct MMU {
    pub free_list: [bool; PAGE_NUMBER as usize], // Physical frames; each frame is taken whenever a process allocates it for itself
    pub allocations: HashMap<Addr, usize>, // Keep track of allocated ram blobs (with size) for dealloc and access/info
}

impl MMU {
    pub fn new_init() -> Self {
        MMU {
            free_list: [false; PAGE_NUMBER as usize],
            // used_list: vec![],
            allocations: HashMap::new(),
        }
    }

    pub fn free_bytes(&self) -> usize {
        self.free_list
            .map(|s| if s { MEM_CTXT.page_size } else { 0 })
            .into_iter()
            .sum::<usize>()
    }

    pub fn dbg(&self) {
        println!("[MMU]");

        let free_bytes = self.free_bytes();
        let total = println!(
            " - Free space: {}B over {}B ({:.3}% available)",
            free_bytes,
            PHYS_TOTAL,
            free_bytes / PHYS_TOTAL * 100
        );
        println!("")
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

    pub fn dbg(&self) {
        println!("---------");
        self.ram.dbg();
        self.mmu.dbg();
        println!("---------");
    }

    /// Get the segment type of word at `addr`
    pub fn get_segment_of(&self, addr: Addr) -> Segment {
        if addr >= self.ram.stack.base && addr < self.ram.stack._end_cap() {
            Segment::Stack
        } else {
            Segment::Neutral // = Heap
        }
    }

    /// Get the page of word at `addr`
    pub fn get_page_of(&self, addr: Addr) -> Option<&Page> {
        self.ram.get_page_of_addr(addr)
    }

    // All access operations are physical-level //
    // and thus should be invoked after translation process //
    // i.e.: `addr` is a physical address //

    fn read_at<T>(&self, addr: Addr) -> MemResult<&[Byte]> {
        Ok(self
            .ram
            ._in
            .get((addr / (MEM_CTXT.page_size as u32)) as usize)
            .ok_or(Fault::_from(FaultType::AddrOutOfRange(addr)))?
            .read::<T>(addr))
    }

    /// Reads word at `addr`, checking if this word is allocated yet.
    pub fn _read_at_checked<T>(&self, addr: Addr) -> Option<MemResult<&[u8]>> {
        self.mmu.allocations.get(&addr)?;

        Some(self.read_at::<T>(addr))
    }

    /// Writes a singular byte at `addr`.
    /// Mostly used in a non-allocation-guarded context.
    fn _write_at_addr<T>(&mut self, addr: Addr, bytes: &[u8]) -> EmptyO {
        // e.g.: Write no-alloc
        self.ram
            ._in
            .get_mut((addr / (MEM_CTXT.page_size as u32)) as usize)
            .ok_or(Fault::_from(FaultType::AddrOutOfRange(addr)))
            .ok()?
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
        let page_at_addr = self.get_page_of(addr); // Physical frame containing that address
    }

    // Checks for no conflict with stack and other already present allocations
    pub fn _alloc_checked(&mut self, addr: Addr, n: usize) {
        if self.mmu.allocations.get(&addr).is_some() {
            ()
        } else if let Segment::Stack = self.get_segment_of(addr) {
            ()
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
        if let Segment::Neutral = self.get_segment_of(addr) {
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
    pub fn _pop(&mut self) -> MemResult<&Byte> {
        self.ram.stack._pop_sp()?; // Pop occurs before to prevent reference conflict
        let r = self.read_at::<Byte>(self.ram.stack.sp)?;
        Ok(&r[0])
    }
}

/// Machine ram context, referencing bitmode, several paging masks and information about the paging machine

#[cfg(test)]
mod tests {
    #[cfg(feature = "bit32")]
    mod inner {
        use crate::mem::{Memory, config::MEM_CTXT};

        #[test]
        // Test miscellaneous mem values against their correspondant in cfg to check for init errors
        fn test_against_cfg() {
            let mem = Memory::new();
            assert_eq!(
                size_of_val(&mem.mmu.free_list),
                MEM_CTXT.page_count as usize
            );
        }
    }

    #[cfg(feature = "bit8")]
    mod inner {
        use crate::mem::{Memory, config::MEM_CTXT};

        #[test]
        fn test_against_cfg() {
            let mem = Memory::new();
            assert_eq!(
                size_of_val(&mem.mmu.free_list),
                MEM_CTXT.page_count as usize
            );
        }
    }
}
