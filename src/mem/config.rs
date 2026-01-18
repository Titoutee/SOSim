use crate::mem::{
    BitMode,
    addr::Addr,
    config::bitmode::{
        _BIT_MODE, _LVL_MASK, _OFF_MASK, _PAGE_COUNT, _PAGE_SIZE, _PHYS_BITW, _PT_LEVELS,
        _V_ADDR_LVL_LEN, _V_ADDR_OFF_LEN,
    },
};
use serde::Deserialize;

/// preset according to the bitmode.
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct MemContext {
    pub bitmode: BitMode,
    pub lvl_mask: u64,
    pub off_mask: u64,
    pub page_size: usize, // Page size is constant across vmem and pmem
    pub page_count: u32,
    // multilevel: bool,
    pub pt_levels: u8,
    pub v_addr_lvl_len: u8,
    pub v_addr_off_len: u8,
    pub phys_bitw: u8,
    pub stack_base: Addr,
    pub stack_sz: Addr,
    // Not in config //
    pub physical_mem_sz: u64, // In words
}

pub static MEM_CTXT: MemContext = MemContext {
    bitmode: _BIT_MODE,
    lvl_mask: _LVL_MASK,
    off_mask: _OFF_MASK,
    page_size: _PAGE_SIZE,
    page_count: _PAGE_COUNT,
    pt_levels: _PT_LEVELS,
    v_addr_lvl_len: _V_ADDR_LVL_LEN,
    v_addr_off_len: _V_ADDR_OFF_LEN,
    phys_bitw: _PHYS_BITW,
    stack_base: _STACK_BASE,
    stack_sz: _STACK_SZ,
    physical_mem_sz: BIT_BASE.pow(_PHYS_BITW as u32),
};

/////////// main ///////////
pub const _PTE_LEN: u8 = 64;

// Most parts of the PTE are ignored in this project
// The unary mask will be used for `present` & `writable` at most.
// /!\ The PTE format is still 64b, to stick to reality.

//////////// Masks are used in to parse the format in this exact order, as they are listed here in the LSB->MSB way ///////////

// pub const PTE_P_MASK: u64 = 0b1;
// pub const PTE_W_MASK: u64 = 0b1;
// pub const PTE_R_MASK: u64 = 0b1;

// No, stack is just really for stack overflow testing, let's keep it smol :D

pub const _STACK_BASE: Addr = 0;
pub const _STACK_SZ: Addr = 64;
const BIT_BASE: u64 = 2;

// (!)
// 8-bit
#[cfg(feature = "bit8")]
pub mod bitmode {
    use super::BitMode;
    pub const _BIT_MODE: BitMode = BitMode::Bit8;
    pub const _LVL_MASK: u64 = 0b111111111;
    pub const _OFF_MASK: u64 = 0b111111111111;
    pub const _PAGE_SIZE: usize = 4 * 1024;
    pub const _PAGE_COUNT: u32 = 512;
    // pub const _MULTI_LEVEL: bool = true;
    pub const _PT_LEVELS: u8 = 4;
    pub const _V_ADDR_LVL_LEN: u8 = 9;
    pub const _V_ADDR_OFF_LEN: u8 = 12;
    pub const _PHYS_BITW: u8 = 8;
    pub const _PTE_PHYS_ADDR_FR_MASK: u32 = 0b11111111; // 8b
    // pub const _STACK_BASE: Addr = 0;
    // pub const _STACK_SZ: Addr = 512; // Addr for address arithmetic easiness
    pub type Addr = u32;
    pub type Vaddr = u32;
}

// (!)
// 32-bit
#[cfg(feature = "bit32")]

pub mod bitmode {
    use super::BitMode;
    pub const _BIT_MODE: BitMode = BitMode::Bit32;
    pub const _LVL_MASK: u64 = 0b111111111;
    pub const _OFF_MASK: u64 = 0b111111111111;
    pub const _PAGE_SIZE: usize = 4 * 1024;
    pub const _PAGE_COUNT: u32 = 512;
    // pub const _MULTI_LEVEL: bool = true;
    pub const _PT_LEVELS: u8 = 4;
    pub const _V_ADDR_LVL_LEN: u8 = 9;
    pub const _V_ADDR_OFF_LEN: u8 = 12;
    pub const _PHYS_BITW: u8 = 32;
    pub const _PTE_PHYS_ADDR_FR_MASK: u32 = 0b11111111111111111111111111111111; // 32b 
    // pub const _STACK_BASE: Addr = 0;
    // pub const _STACK_SZ: Addr = 512; // Addr for address arithmetic easiness
    pub type Addr = u32;
    pub type Vaddr = u32;
}

#[cfg(feature = "bit32")]
pub const JSON_PREFIX: &str = "32";
#[cfg(feature = "bit8")]
pub const JSON_PREFIX: &str = "8";
