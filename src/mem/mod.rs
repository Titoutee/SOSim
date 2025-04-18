//! Interface for mem structures, including a simulated DRAM bank, CPU registers, paging mechanisms, ...
pub mod config;
pub mod paging;
pub mod addr;

/*pub*/ use addr::{Addr, VAddr, _VAddrRawCtxt};
/*pub*/ use paging::{PageTable, RawPTEntry, FullPTEntry};

pub enum BitMode {
    Bit8,
    Bit16,
    Bit32,
    Bit64,
}