//! Interface for mem structures, including a simulated DRAM bank, CPU registers, paging mechanisms, ...

pub mod addr;
pub mod config;
pub mod stack;

use std::fs;

use addr::{Addr, VirtualAddress};
use config::bitmode;
use serde::Deserialize;
use serde_json;

#[cfg(feature = "bit64")]
pub const JSON_PREFIX: &str = "64";
#[cfg(feature = "bit32")]
pub const JSON_PREFIX: &str = "32";
#[cfg(feature = "bit16")]
pub const JSON_PREFIX: &str = "16";
#[cfg(feature = "bit8")]
pub const JSON_PREFIX: &str = "8";

// type ParseResult = Result<(), String>;

// /*pub*/ use addr::{Addr, VAddr, _VAddrRawCtxt};
// /*pub*/ use paging::{PageTable, RawPTEntry, FullPTEntry};

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum BitMode {
    Bit8,
    Bit16,
    Bit32,
    Bit64,
}

#[derive(Debug)]
pub struct MemoryRegion {
    pub start: Addr,
    pub size: usize,
    pub name: String,
    // pub is_guard: bool // guarding is located within paging mechanisms
}

#[derive(Debug)]
pub struct Ram {
    pub memory: Vec<u8>, // Mem words are 8-bit wide
    pub size: usize,
}

impl Ram {}

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
        use bitmode::*;

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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MemContext;

    #[test]
    #[cfg(feature = "bit64")]
    fn from_js_64b() {
        use super::JSON_PREFIX;

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
