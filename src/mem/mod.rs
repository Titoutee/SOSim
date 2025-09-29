//! Interface for mem structures, including a simulated DRAM bank, CPU registers, paging mechanisms, ...

pub mod addr;
pub mod config;

use crate::{fault::{Fault, FaultType}};
use std::{collections::HashMap, fs};
use config::{bitmode::{Addr}};
use serde::Deserialize;
use serde_json;
pub type Byte = u8;

#[derive(Debug)]
pub enum RegionType {
    Heap,
    Stack,
    Neutral,
}

#[derive(Debug)]
pub struct MemRegion { // Which can apply to both address spaces (virtual) and physical mem
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
pub struct MemBlob(usize, RegionType);
type EmptyO = Option<()>;
type EmptyR = Result<(), ()>;
type RamResult<T> = Result<T, Fault>;

/// MMU made of a RAM bank model, acting as a mere allocation pool without any time-sync and alignment constraint.
/// 
/// *(The MMU naming can lead to confusion here, as it also includes the usable memory itself, but we consider it's just accessible
/// remotely from the MMU from a hardware perspective)*
/// 
/// Internally, the bank is made of a capacity-cap-ed vector (of capacity **2^`_PHYS_BITW`**).
/// 
/// **WARNING**: if using a custom config, this scalar should be kept relatively low.
#[derive(Debug)]
pub struct MMU<'a> {
    pub context: &'a MemContext,
    pub memory: Vec<u8>, // Mem words are 8-bit wide
    pub allocations: HashMap<Addr, MemBlob>, // Keep track of allocated memory blobs (with size)
}

impl<'a> MMU<'a> {
    pub fn new(memctxt: &'a MemContext) -> Self {
        Self {
            context: memctxt,
            memory: Vec::with_capacity(memctxt.mem_size),
            allocations: HashMap::new(),
        }
    }
    // All these access operations are physical-level. //

    fn _at(&self, addr: Addr) -> RamResult<&Byte> {
        self.memory.get(addr as usize).ok_or(Fault::from(FaultType::AddrOutOfRange(addr)))
    }

    fn _at_mut(&mut self, addr: Addr) -> RamResult<&mut Byte> {
        self.memory.get_mut(addr as usize).ok_or(Fault::from(FaultType::AddrOutOfRange(addr)))
    }   
    /// Reads word at `addr`
    pub fn read_at_addr(&self, addr: Addr) -> RamResult<&Byte> {
        self._at(addr)
    }

    // Reads word at `addr`, checking if this word is allocated yet.
    pub fn read_at_addr_checked(&self, addr: Addr) -> Option<RamResult<&u8>> {
        self.allocations.get(&addr)?;
        Some(self.read_at_addr(addr))
    }

    // Writes a singular byte at `addr`.
    pub fn write_at_addr(&mut self, addr: Addr, byte: Byte) -> EmptyO { // e.g.: Write no-alloc
        let a = self._at_mut(addr).ok()?;
        *a = byte;
        Some(())
    }
    
    pub fn write_at_addr_checked(&mut self, addr: Addr, byte: Byte) -> EmptyO {
        self.allocations.get(&addr).is_none().then(||())?;
        self.write_at_addr(addr, byte);
        Some(())
    }

    pub fn dealloc(&mut self, addr: Addr) -> EmptyO {
        self.allocations.remove(&addr).map(|_| ())
    }
}

/// Machine memory context, referencing bitmode, several paging masks and information about the paging machine
/// preset according to the bitmode.
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct MemContext {
    bitmode: BitMode,
    lvl_mask: u64,
    off_mask: u64,
    page_size: u32,
    page_count: u16,
    // multilevel: bool,
    pt_levels: u8,
    v_addr_lvl_len: u8,
    v_addr_off_len: u8,
    phys_bitw: u8,
    mem_size: usize,
}

impl MemContext {
    // /!\
    fn _new(
        bitmode: BitMode,
        lvl_mask: u64,
        off_mask: u64,
        page_size: u32,
        page_count: u16,
        //multilevel: bool,
        pt_levels: u8,
        v_addr_lvl_len: u8,
        v_addr_off_len: u8,
        phys_bitw: u8,
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
            mem_size: 2 ^ phys_bitw as usize,
        }
    }

    pub fn new() -> Self {
        MemContext::_from_bit_mode_compiled()
    }

    /// Parse a memory context from a json configuration referencing the different fields of `MemContext`.
    ///
    /// Substitution to `_from_bit_mode_compiled`.
    pub fn from_json(path: &str) -> Result<MemContext, serde_json::Error> {
        let json: String = fs::read_to_string(path).unwrap();
        let ctxt: MemContext = serde_json::from_str(&json)?;
        Ok(ctxt)
    }

    /// Use the conditionally-compiled paging constants to returned a fresh, pre-configured memory context.
    ///
    /// Relevant magic values can be found at `./config.rs`.
    pub const fn _from_bit_mode_compiled() -> Self {
        use config::bitmode::*;

        Self {
            bitmode: _BIT_MODE,
            lvl_mask: _LVL_MASK,
            off_mask: _OFF_MASK,
            page_size: _PAGE_SIZE,
            page_count: _PAGE_COUNT,
            // multilevel: _MULTI_LEVEL,
            pt_levels: _PT_LEVELS,
            v_addr_lvl_len: _V_ADDR_LVL_LEN,
            v_addr_off_len: _V_ADDR_OFF_LEN,
            phys_bitw: _PHYS_BITW,
            mem_size: _MEM_SIZE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MemContext;
    use super::config::JSON_PREFIX;
    #[test]
    #[cfg(feature = "bit64")]
    fn from_js_64b() {
        let memctxt = MemContext::new(); // Set for 64b
        let path = format!("bitmodes/{}b.json", JSON_PREFIX);
        // println!("{}", path);

        let from_js = MemContext::from_json(&path).unwrap();
        assert_eq!(memctxt, from_js);
    }

    #[test]
    #[cfg(feature = "bit32")]
    fn from_js_32b() {
        use super::JSON_PREFIX;
        println!("{}", JSON_PREFIX);
        let memctxt = MemContext::new(); // Set for 32b

        let from_js = MemContext::from_json(&format!("bitmodes/{}b.json", JSON_PREFIX)).unwrap();
        assert_eq!(memctxt, from_js);
        //
    }

    #[test]
    #[cfg(feature = "bit16")]
    fn from_js_16b() {
        use super::JSON_PREFIX;

        let memctxt = MemContext::new(); // Set for 16b

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
