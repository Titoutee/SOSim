use super::{addr::{self, Addr}, config::{self, MemContext}};
use config::{PTE_PHYS_ADDR_MASK};

pub struct PageTable {
    // PTEs are hard-indexed, which means the index part (9 bit for 64-bit v-addr) in the v-addr is directly used to access the appropriate PTE
    arr: [RawPTEntry; 512], // (!)
}

pub struct RawPTEntry {
    bitset: u64, // std bitset for PTE format in x86_64 is 64 bit long
}

impl RawPTEntry {
    pub fn new(bitset: u64) -> Self {
        Self {bitset}
    }

    fn from_full(full_PTE: FullPTEntry) -> Self {
        todo!()
    }
}

pub struct FullPTEntry {
    present: bool,
    write: bool,
    read: bool,
    phys_addr: u64,
}

impl FullPTEntry {
    fn from_raw(r: RawPTEntry) -> Self {
        let bits = r.bitset;
        
        let present = (bits & 0b1).to_bool();
        let write = ((bits >> 1) & 0b1).to_bool();
        let read = ((bits >> 2) & 0b1).to_bool();
        let phys_addr = (bits >> 3) & PTE_PHYS_ADDR_MASK;

        Self {
            present, write, read, phys_addr,
        }
    }
}

pub trait UNSIGNED64Ext {
    fn to_bool(&self) -> bool;
}

impl UNSIGNED64Ext for u64 {
    fn to_bool(&self) -> bool {
        if *self == 0 {false} else {true}
    }
}