use super::addr::Addr;

pub const PAGE_SIZE: u16 =  4_096; // Typical page size for most platforms. (!)
pub const V_ADDR_SPACE_BITW: u8 = 32; // Virtual address space size. (!)
pub const P_ADDR_SPACE_BITW: u8 = 64; // Physical mem size. (!)
pub const ENT_COUNT: u16 = 512; // Number of page table entries within a page table level. (!)

pub struct PageTable {
    inner: [PTEntry; 512],
}

// /!\ Alignment
#[repr(C)]
pub struct PTEntry {
    phys_frame_addr: Addr,
    r: bool,
    w: bool,
}