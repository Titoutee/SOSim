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

trait _From<T> {
    fn _from(t: T) -> Self;
}

trait _Into<T> {
    fn _into(&self) -> T;
}

impl _From<u32> for bool {
    fn _from(t: u32) -> Self {
        if t == 0 {false} else {true}
    }
}

impl _Into<u32> for bool {
    fn _into(&self) -> u32 {
        if *self { 0b1 } else { 0b0 }
    }
}

