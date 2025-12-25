//! Interface for mem structures, including a simulated DRAM bank, CPU registers, paging mechanisms, ...

pub mod addr;
pub mod config;
pub mod paging;

use crate::{
    fault::{Fault, FaultType},
    mem::{
        config::{STACK_BASE, STACK_SZ},
        paging::Page,
    },
};
use config::bitmode::Addr;
use num::pow::Pow;
use serde::Deserialize;
use serde_json;
use std::{collections::HashMap, fs, ops::Add};
pub type Byte = u8;

const BIT_BASE: u64 = 2;

#[derive(Debug)]
pub enum RegionType {
    Heap,
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
    sp: u64,
    cap: u64,
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
/// initialised according to (pre-)defined memory context settings, the stack and heap positions within main memory, etc...
#[derive(Debug)]
pub struct Ram {
    pub _in: Vec<u8>,
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
    pub active: Vec<bool>, // Track keeping of allocated/active pages in main memory: at index `i` is `true` if the `i`th page of main memory is allocated
    pub allocations: HashMap<Addr, MemBlob>, // Keep track of allocated ram blobs (with size) for dealloc and access/info
}

impl MMU {
    pub fn new_init(page_count: usize) -> Self {
        MMU {
            active: vec![false; page_count],
            allocations: HashMap::new(),
        }
    }
}

/// A SRAM bank model, acting as a mere allocation pool without any time-sync and alignment constraint, allied with memory context and an [MMU][MMU].
///
#[derive(Debug)]
pub struct Memory<'a> {
    pub mmu: MMU,
    pub context: &'a MemContext,
    pub ram: Ram, // Mem words are 8-bit wide
    pub free_list: Vec<Page>,
}

impl<'a> Memory<'a> {
    pub fn new(memctxt: &'a MemContext) -> Self {
        let b: u8 = 2;
        let stack = Stack {
            base: memctxt.stack_base as u64,
            sz: 0,
            cap: memctxt.stack_sz as u64,
            sp: 0,
        };
        let free_list = Vec::from_iter(std::iter::repeat(Page()).take(memctxt.page_count as usize));
        Self {
            mmu: MMU::new_init(memctxt.page_count),
            context: memctxt,
            ram: Ram::new(b.pow(memctxt.phys_bitw as u32) as usize, stack),
            free_list,
        }
    }

    // All access operations are physical-level //
    // and should be invoked after translation process //

    fn _at(&self, addr: Addr) -> MemResult<&Byte> {
        self.ram
            ._in
            .get(addr as usize)
            .ok_or(Fault::_from(FaultType::AddrOutOfRange(addr)))
    }

    fn _at_mut(&mut self, addr: Addr) -> MemResult<&mut Byte> {
        self.ram
            ._in
            .get_mut(addr as usize)
            .ok_or(Fault::_from(FaultType::AddrOutOfRange(addr)))
    }
    /// Reads word at `addr`
    pub fn _read_at_addr(&self, addr: Addr) -> MemResult<Byte> {
        self._at(addr).map(|x| *x) // u8s are easy to copy :D
    }

    /// Reads word at `addr`, checking if this word is allocated yet.
    pub fn _read_at_addr_checked(&self, addr: Addr) -> Option<MemResult<u8>> {
        self.mmu.allocations.get(&addr)?;
        Some(self._read_at_addr(addr))
    }

    /// Writes a singular byte at `addr`.
    /// Mostly used in a non-allocation-guarded context.
    pub fn _write_at_addr(&mut self, addr: Addr, byte: Byte) -> EmptyO {
        // e.g.: Write no-alloc
        let a = self._at_mut(addr).ok()?;
        *a = byte;
        Some(())
    }

    /// Writes a singular byte at `addr`, checking if this word is allocated yet.
    /// To be used in an allocation-guarded context.
    pub fn _write_at_addr_checked(&mut self, addr: Addr, byte: Byte) -> EmptyO {
        self.mmu.allocations.get(&addr).is_none().then(|| ())?;
        self._write_at_addr(addr, byte);
        Some(())
    }

    pub fn _dealloc(&mut self, addr: Addr) -> EmptyO {
        self.mmu.allocations.remove(&addr).map(|_| ())
    }

    // Stack
    pub fn _push(&mut self, byte: Byte) -> MemResult<()> {
        *self._at_mut(self.ram.stack.sp)? = byte;
        self.ram.stack._push_sp();
        Ok(())
    }

    pub fn _pop(&mut self) -> MemResult<u8> {
        let r = *self._at(self.ram.stack.sp)?;
        self.ram.stack._pop_sp()?;
        Ok(r)
    }
    //
}

/// Machine ram context, referencing bitmode, several paging masks and information about the paging machine
/// preset according to the bitmode.
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct MemContext {
    pub bitmode: BitMode,
    pub lvl_mask: u64,
    pub off_mask: u64,
    pub page_size: u32, // Page size is constant across vmem and pmem
    pub page_count: usize,
    // multilevel: bool,
    pub pt_levels: u8,
    pub v_addr_lvl_len: u8,
    pub v_addr_off_len: u8,
    pub phys_bitw: u8,
    pub stack_base: usize,
    pub stack_sz: usize,
    // Not in config //
    pub physical_mem_sz: u64, // In words
}

impl MemContext {
    // /!\
    fn _new(
        bitmode: BitMode,
        lvl_mask: u64,
        off_mask: u64,
        page_size: u32,
        page_count: usize,
        //multilevel: bool,
        pt_levels: u8,
        v_addr_lvl_len: u8,
        v_addr_off_len: u8,
        phys_bitw: u8,
        stack_base: usize,
        stack_sz: usize,
    ) -> Self {
        Self {
            bitmode,
            lvl_mask,
            off_mask,
            page_size,
            page_count,
            //multilevel,
            pt_levels,
            v_addr_lvl_len,
            v_addr_off_len,
            phys_bitw,
            stack_base,
            stack_sz,
            physical_mem_sz: BIT_BASE.pow(phys_bitw as u32),
        }
    }

    pub fn new() -> Self {
        MemContext::_from_bit_mode_compiled()
    }

    /// Parse a ram context from a json configuration referencing the different fields of `MemContext`.
    ///
    /// Substitution to `_from_bit_mode_compiled`.
    pub fn from_json(path: &str) -> Result<MemContext, serde_json::Error> {
        let json: String = fs::read_to_string(path).unwrap();
        let mut ctxt: MemContext = serde_json::from_str(&json)?;
        ctxt.physical_mem_sz = BIT_BASE.pow(ctxt.physical_mem_sz as u32);
        Ok(ctxt)
    }

    /// Use the conditionally-compiled paging constants to returned a fresh, pre-configured ram context.
    ///
    /// Relevant magic values can be found at `./config.rs`.
    pub fn _from_bit_mode_compiled() -> Self {
        use config::bitmode::*;

        Self {
            bitmode: _BIT_MODE,
            lvl_mask: _LVL_MASK,
            off_mask: _OFF_MASK,
            page_size: _PAGE_SIZE,
            page_count: _PAGE_COUNT,
            pt_levels: _PT_LEVELS,
            v_addr_lvl_len: _V_ADDR_LVL_LEN,
            v_addr_off_len: _V_ADDR_OFF_LEN,
            phys_bitw: _PHYS_BITW,
            stack_base: STACK_BASE,
            stack_sz: STACK_SZ,
            physical_mem_sz: BIT_BASE.pow(_PHYS_BITW as u32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MemContext;
    pub use super::config::JSON_PREFIX;

    #[test]
    #[cfg(feature = "bit32")]
    fn from_js_32b() {
        println!("{}", JSON_PREFIX);
        let memctxt = MemContext::new(); // Set for 32b

        let from_js = MemContext::from_json(&format!("bitmodes/{}b.json", JSON_PREFIX)).unwrap();
        assert_eq!(memctxt, from_js);
        //
    }

    #[test]
    #[cfg(feature = "bit8")]
    fn from_js_8b() {
        use super::JSON_PREFIX;

        let memctxt = MemContext::new(); // Set for 8b

        let from_js = MemContext::from_json(&format!("bitmodes/{}b.json", JSON_PREFIX)).unwrap();
        assert_eq!(memctxt, from_js);
        //
    }
}
