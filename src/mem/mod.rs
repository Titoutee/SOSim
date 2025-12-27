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
use config::bitmode::Addr;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json;
use std::{collections::HashMap, fs};

use crate::lang::{Byte, Struct};

#[derive(Debug)]
pub enum RegionType {
    Stack,
    Neutral,
}

#[derive(Debug)]
pub struct MemRegion {
    // Which can apply to both address spaces (virtual) and physical mem
    start: Addr,
    size: usize,
    typ: RegionType,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum BitMode {
    Bit8,
    Bit16,
    Bit32,
    Bit64,
}

#[derive(Debug)]
pub struct MemBlob(Addr, RegionType, usize); // (addr, regiontype, size)
type EmptyO = Option<()>;
type EmptyR = Result<(), ()>;
type MemResult<T> = Result<T, Fault>;

#[derive(Debug)]
pub struct Stack {
    base: Addr,
    sz: usize,
    sp: Addr,
    cap: Addr,
}

impl Stack {
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
}

#[derive(Debug)]
pub struct MMU {
    pub free_list: Vec<Page>,
    pub used_list: Vec<Page>,                // Physical frames
    pub allocations: HashMap<Addr, MemBlob>, // Keep track of allocated ram blobs (with size) for dealloc and access/info
}

impl MMU {
    pub fn new_init() -> Self {
        let mut base = 0;
        let free_list = Vec::from_iter(std::iter::from_fn(move || {
            let res = if base >= MEM_CTXT.physical_mem_sz as Addr {
                None
            } else {
                const PGSZ: u32 = MEM_CTXT.page_size as u32;
                Some(Page {
                    data: [0; PGSZ as usize],
                    addr: base,
                })
            };

            base += MEM_CTXT.page_size as u32;
            res
        }));

        MMU {
            free_list,
            used_list: vec![],
            allocations: HashMap::new(),
        }
    }
}

/// A SRAM bank model, acting as a mere allocation pool allied with memory context and an [MMU][MMU].
///
#[derive(Debug)]
pub struct Memory<'a> {
    pub mmu: MMU,
    pub context: &'a MemContext,
    pub ram: Ram,
}

impl<'a> Memory<'a> {
    pub fn new() -> Self {
        let stack = Stack {
            base: MEM_CTXT.stack_base as Addr,
            sz: 0,
            cap: MEM_CTXT.stack_sz as Addr,
            sp: 0,
        };

        Self {
            mmu: MMU::new_init(),
            context: &MEM_CTXT,
            ram: Ram::new(MEM_CTXT.page_count as usize, stack), // Allocate for `page_count` pages
        }
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
    pub fn _alloc(&mut self, addr: Addr, n: usize) {}

    pub fn _dealloc(&mut self, addr: Addr) -> EmptyO {
        self.mmu.allocations.remove(&addr).map(|_| ())
    }

    // Stack
    pub fn _push(&mut self, byte: Byte) -> MemResult<()> {
        self._write_at_addr::<Byte>(self.ram.stack.sp, &[byte]);
        self.ram.stack._push_sp();
        Ok(())
    }

    // Always pop a singular byte from stack using `_pop`
    pub fn _pop(&mut self) -> MemResult<&Byte> {
        self.ram.stack._pop_sp()?; // Pop occurs before to prevent reference conflict
        let r = self.read_at::<Byte>(self.ram.stack.sp)?;
        Ok(&r[0])
    }
    //
}

/// Machine ram context, referencing bitmode, several paging masks and information about the paging machine

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "bit32")]
    fn mem_new_32b() {
        use crate::mem::MEM_CTXT;
        use crate::mem::Memory;

        let mem: Memory<'_> = Memory::new();
        // println!("{:?}", mem.free_list);
        let itr = mem.mmu.free_list.windows(2);
        for i in itr {
            let a = i[0].addr;
            let b = i[1].addr;
            assert_eq!(b - a, MEM_CTXT.page_size as u32);
        }
    }

    #[test]
    #[cfg(feature = "bit8")]
    fn mem_new_8b() {
        use crate::mem::MEM_CTXT;
        use crate::mem::Memory;

        let mem = Memory::new();
        // println!("{:?}", mem.free_list);
        let itr = mem.free_list.windows(2);
        for i in itr {
            let a = i[0].0;
            let b = i[1].0;
            assert_eq!(b - a, MEM_CTXT.page_size as u64);
        }
    }
}
