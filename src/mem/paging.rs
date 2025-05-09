// PTE format does not exactly match the x86_64 standard, as only present, write, read bits and the address payload is are serialized
// into the 64b bitset.

use super::{
    addr::{self, Addr},
    config::{self, MemContext},
};
use config::PTE_PHYS_ADDR_MASK;
// use num::{BigUint, PrimInt, Integer};

pub struct RawPTEntry {
    bitset: u64, // std bitset for PTE format in x86_64 is 64 bit long
}

impl RawPTEntry {
    pub fn new(bitset: u64) -> Self {
        Self { bitset }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct FullPTEntry {
    present: bool,
    write: bool,
    read: bool,
    phys_addr: u64,
}

impl FullPTEntry {
    pub fn new(present: bool, write: bool, read: bool, phys_addr: u64) -> Self {
        Self {
            present,
            write,
            read,
            phys_addr
        }
    }   
}

trait _From<T> {
    fn _from(t: T) -> Self;
}

trait _Into<T> {
    fn _into(&self) -> T;
}

//// u64 <-> bool 
//// (!) extend to T: PrimInt
impl _From<u64> for bool {
    fn _from(t: u64) -> Self {
        if t == 0 {return false;}true
    }
}

impl _Into<u64> for bool {
    fn _into(&self) -> u64 {
        if *self {return 0b1} 0b0
    }
}

//// RawPTE <-> FullPTE
impl From<RawPTEntry> for FullPTEntry {
    fn from(r: RawPTEntry) -> Self {
        let bits = r.bitset;

        let present = <bool as _From<u64>>::_from(bits & 0b1);
        let write = <bool as _From<u64>>::_from((bits >> 1) & 0b1);
        let read = <bool as _From<u64>>::_from((bits >> 2) & 0b1);
        let phys_addr = (bits >> 3) & PTE_PHYS_ADDR_MASK;

        Self {
            present,
            write,
            read,
            phys_addr,
        }
    }
}
////

pub struct PageTable {
    // PTEs are hard-indexed, which means the index part (9 bit for 64-bit v-addr) in the v-addr is directly used to access the appropriate PTE
    arr: [RawPTEntry; 512], // (!)
}

#[cfg(test)]
mod tests {
    use super::{FullPTEntry, RawPTEntry, _From, _Into};

    #[test]
    fn bool_from_u64() {
        let t = <bool as _From<u64>>::_from(1);
        let f = <bool as _From<u64>>::_from(0);
        assert!(t && !f);
    }

    #[test]
    fn u64_from_bool() {
        let nt = true._into();
        let nf = false._into();
        assert_eq!(nt, 0b1);
        assert_eq!(nf, 0);
    }

    #[test]
    fn fpte_from_raw_pwr() {
        let raw = 0b10111; // Auto extension
        let rawpte = RawPTEntry::new(raw);
        let fpte = FullPTEntry::from(rawpte);
        let comp = FullPTEntry::new(true, true, true, 0b10);
        assert_eq!(comp, fpte);
    }

    #[test]
    fn fpte_from_raw_p() {
        let raw = 0b10001; // Auto extension
        let rawpte = RawPTEntry::new(raw);
        let fpte = FullPTEntry::from(rawpte);
        let comp = FullPTEntry::new(true, false, false, 0b10);
        assert_eq!(comp, fpte);
    }

    #[test]
    fn fpte_from_raw_pr() {
        let raw = 0b10101; // Auto extension
        let rawpte = RawPTEntry::new(raw);
        let fpte = FullPTEntry::from(rawpte);
        let comp = FullPTEntry::new(true, false, true, 0b10);
        assert_eq!(comp, fpte);
    }
}
