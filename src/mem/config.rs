use crate::mem::BitMode;

// main
pub const _PTE_LEN: u8 = 64;
// pub const UNARY_MASK: u8 = 0b1;

// Most parts of the PTE are ignored in this project
// The unary mask will be used for `present` & `writable` at most.
// /!\ The PTE format is still 64b, to stick to reality.

//// Masks are used in to parse the format in this exact order, as they are listed here in the LSB->MSB way.

// pub const PTE_P_MASK: u64 = 0b1;
// pub const PTE_W_MASK: u64 = 0b1;
// pub const PTE_R_MASK: u64 = 0b1;
pub const PTE_PHYS_ADDR_MASK: u64 = 0b1111111111111111111111111111111111111111111111111111;

// 8-bit
#[cfg(feature = "bit8")]
pub mod bitmode {
    use super::BitMode;
    pub const _BIT_MODE: BitMode = BitMode::Bit32;
    pub const _LVL_MASK: u16 = 0b111111111;
    pub const _OFF_MASK: u16 = 0b111111111111;
    pub const _PAGE_SIZE: u32 = 4 * 1024;
    pub const _PAGE_COUNT: u16 = 512;
    pub const _MULTI_LEVEL: bool = true;
    pub const _PT_LEVELS: u8 = 4;
    pub const _V_ADDR_LVL_LEN: u8 = 9;
    pub const _V_ADDR_OFF_LEN: u8 = 12;
    pub const _PHYS_BITW: u8 = 128;
}

// 16-bit
#[cfg(feature = "bit16")]
pub mod bitmode {
    use super::BitMode;
    pub const _BIT_MODE: BitMode = BitMode::Bit32;
    pub const _LVL_MASK: u16 = 0b111111111;
    pub const _OFF_MASK: u16 = 0b111111111111;
    pub const _PAGE_SIZE: u32 = 4 * 1024;
    pub const _PAGE_COUNT: u16 = 512;
    pub const _MULTI_LEVEL: bool = true;
    pub const _PT_LEVELS: u8 = 4;
    pub const _V_ADDR_LVL_LEN: u8 = 9;
    pub const _V_ADDR_OFF_LEN: u8 = 12;
    pub const _PHYS_BITW: u8 = 128;
}

// 32-bit
#[cfg(feature = "bit32")]
pub mod bitmode {
    use super::BitMode;
    pub const _BIT_MODE: BitMode = BitMode::Bit64;
    pub const _LVL_MASK: u16 = 0b111111111;
    pub const _OFF_MASK: u16 = 0b111111111111;
    pub const _PAGE_SIZE: u32 = 4 * 1024;
    pub const _PAGE_COUNT: u16 = 0;
    pub const _MULTI_LEVEL: bool = true;
    pub const _PT_LEVELS: u8 = 4;
    pub const _V_ADDR_LVL_LEN: u8 = 9;
    pub const _V_ADDR_OFF_LEN: u8 = 12;
    pub const _PHYS_BITW: u8 = 128;
}

// 64-bit
#[cfg(feature = "bit64")]
pub mod bitmode {
    use super::BitMode;
    pub const _BIT_MODE: BitMode = BitMode::Bit32;
    pub const _LVL_MASK: u16 = 0b111111111;
    pub const _OFF_MASK: u16 = 0b111111111111;
    pub const _PAGE_SIZE: u32 = 4 * 1024;
    pub const _PAGE_COUNT: u16 = 512;
    pub const _MULTI_LEVEL: bool = true;
    pub const _PT_LEVELS: u8 = 4;
    pub const _V_ADDR_LVL_LEN: u8 = 9;
    pub const _V_ADDR_OFF_LEN: u8 = 12;
    pub const _PHYS_BITW: u8 = 128;
}

#[cfg(feature = "bit64")]
pub use bitmode as paging_consts;
#[cfg(feature = "bit32")]
pub use bitmode as paging_consts;

pub struct MemContext {
    lvl_mask: u16,
    off_mask: u16,
    page_size: u32,
    page_count: u16,
    multilevel: bool,
    pt_levels: u8,
    v_addr_lvl_len: u8,
    v_addr_off_len: u8,
    phys_bitw: u8,
}

impl MemContext {
    // /!\
    fn new(
        lvl_mask: u16,
        off_mask: u16,
        page_size: u32,
        page_count: u16,
        multilevel: bool,
        pt_levels: u8,
        v_addr_lvl_len: u8,
        v_addr_off_len: u8,
        phys_bitw: u8,
    ) -> Self {
        Self {
            lvl_mask,
            off_mask,
            page_size,
            page_count,
            multilevel,
            pt_levels,
            v_addr_lvl_len,
            v_addr_off_len,
            phys_bitw,
        }
    }

    fn from_bit_mode__compiled() -> Self {
        use paging_consts::*;
        Self {
            lvl_mask: _LVL_MASK,
            off_mask: _OFF_MASK,
            page_size: _PAGE_SIZE,
            page_count: _PAGE_COUNT,
            multilevel: _MULTI_LEVEL,
            pt_levels: _PT_LEVELS,
            v_addr_lvl_len: _V_ADDR_LVL_LEN,
            v_addr_off_len: _V_ADDR_OFF_LEN,
            phys_bitw: _PHYS_BITW,
        }
    }
}

//pub trait FromWCtxt<T> {
//    fn from(&self, ctxt: MemContext) -> T;
//}
//
//pub trait IntoWCtxt<T> {
//    fn into(&self, ctxt: MemContext) -> T;
//}
