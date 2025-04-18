use super::BitMode;

// main
pub const _PTE_LEN: u8 = 64;
// pub const UNARY_MASK: u8 = 0b1;

// Most parts of the PTE are ignored in this project
// The unary mask will be used for `present` & `writable` at most.
// /!\ The PTE format is still 64b, to stick to reality.

// Masks are used in to parse the format in this exact order, as they are listed here in the LSB->MSB way.

// pub const PTE_free_os_mask: u8 = 0b111;
pub const PTE_PHYS_ADDR_MASK: u64 = 0b1111111111111111111111111111111111111111111111111111;

// 8-bit
// pub const _8b_lvl_mask: u16 =
// pub const _8b_lvl_mask: u16 =

// 16-bit
// pub const _16b_lvl_mask: u16 =
// pub const _16b_lvl_mask: u16 =

// 32-bit
// pub const _32b_lvl_mask: u16 =
// pub const _32b_lvl_mask: u16 =

// 64-bit
pub const _64_LVL_MASK: u16 = 0b111111111;
pub const _64_OFF_MASK: u16 = 0b111111111111;
pub const _64_PAGE_SIZE: u32 = 4 * 1024;
pub const _64_PAGE_COUNT: u16 = 512;
pub const _64_MULTI_LEVEL: bool = true;
pub const _64_PT_LEVELS: u8 = 4;
pub const _64_V_ADDR_LVL_LEN: u8 = 9;
pub const _64_V_ADDR_OFF_LEN: u8 = 12;
pub const _64_PHYS_BITW: u8 = 128;

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

    fn from_bit_mode(bmode: BitMode) -> Self {
        match bmode {
            BitMode::Bit8 => {
                todo!()
            }
            BitMode::Bit16 => {
                todo!()
            }
            BitMode::Bit32 => {
                todo!()
            }
            BitMode::Bit64 => Self {
                lvl_mask: _64_LVL_MASK,
                off_mask: _64_OFF_MASK,
                page_size: _64_PAGE_SIZE,
                page_count: _64_PAGE_COUNT,
                multilevel: _64_MULTI_LEVEL,
                pt_levels: _64_PT_LEVELS,
                v_addr_lvl_len: _64_V_ADDR_LVL_LEN,
                v_addr_off_len: _64_V_ADDR_OFF_LEN,
                phys_bitw: _64_PHYS_BITW,
            },
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