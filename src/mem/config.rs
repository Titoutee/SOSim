use crate::mem::BitMode;

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

pub const STACK_BASE: usize = 0;
pub const STACK_SZ: usize = 64;

// (!)
// 8-bit
#[cfg(feature = "bit8")]
pub mod bitmode {
    use super::BitMode;
    pub const _BIT_MODE: BitMode = BitMode::Bit8;
    pub const _LVL_MASK: u64 = 0b111111111;
    pub const _OFF_MASK: u64 = 0b111111111111;
    pub const _PAGE_SIZE: u32 = 4 * 1024;
    pub const _PAGE_COUNT: usize = 512;
    // pub const _MULTI_LEVEL: bool = true;
    pub const _PT_LEVELS: u8 = 4;
    pub const _V_ADDR_LVL_LEN: u8 = 9;
    pub const _V_ADDR_OFF_LEN: u8 = 12;
    pub const _PHYS_BITW: u8 = 64;
    pub const _PTE_PHYS_ADDR_FR_MASK: u64 = 0b111111111111111111111111111111111111111111111111111; // 61b
    pub type Addr = u64;
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
    pub const _PAGE_SIZE: u32 = 4 * 1024;
    pub const _PAGE_COUNT: usize = 512;
    // pub const _MULTI_LEVEL: bool = true;
    pub const _PT_LEVELS: u8 = 4;
    pub const _V_ADDR_LVL_LEN: u8 = 9;
    pub const _V_ADDR_OFF_LEN: u8 = 12;
    pub const _PHYS_BITW: u8 = 32;
    pub const _PTE_PHYS_ADDR_FR_MASK: u64 = 0b111111111111111111111111111111111111111111111111111; // 61b 
    pub type Addr = u64;
    pub type Vaddr = u32;
}

#[cfg(feature = "bit32")]
pub const JSON_PREFIX: &str = "32";
#[cfg(feature = "bit8")]
pub const JSON_PREFIX: &str = "8";
