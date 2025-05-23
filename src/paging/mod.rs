// PTE format does not exactly match the x86_64 standard, as only present, write, read bits and the address payload is are serialized
// into the 64b bitset.
mod ext;
pub use ext::{_From, _Into};
use super::mem::{
    addr::{Addr, VirtualAddress},
    config::bitmode::_PTE_PHYS_ADDR_FR_MASK,
};

pub struct PageTable {
    // PTEs are hard-indexed, which means the index part (9 bit for 64-bit v-addr) in the v-addr is directly used to access the appropriate PTE
    arr: [RawPTEntry; 512], // (!)
}

pub struct RawPTEntry {
    bitset: u64, // std bitset for PTE format in x86_64 is 64 bit long
}

impl RawPTEntry {
    pub fn new(bitset: u64) -> Self {
        Self { bitset }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PTEntry {
    present: bool,
    write: bool,
    read: bool,
    phys_frame_addr: u64,
}

impl PTEntry {
    pub fn new(present: bool, write: bool, read: bool, phys_frame_addr: u64) -> Self {
        Self {
            present,
            write,
            read,
            phys_frame_addr,
        }
    }
}


// RawPTE <-> FullPTE
impl From<RawPTEntry> for PTEntry {
    fn from(r: RawPTEntry) -> Self {
        let bits = r.bitset;

        let present = <bool as _From<u64>>::_from(bits & 0b1);
        let write = <bool as _From<u64>>::_from((bits >> 1) & 0b1);
        let read = <bool as _From<u64>>::_from((bits >> 2) & 0b1);
        let phys_frame_addr = (bits >> 3) & _PTE_PHYS_ADDR_FR_MASK;

        Self {
            present,
            write,
            read,
            phys_frame_addr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{_From, _Into, PTEntry, RawPTEntry};

    #[test]
    fn bool_from_u64() {
        let t = <bool as _From<u64>>::_from(1);
        let f = <bool as _From<u64>>::_from(0);
        assert!(t && !f);
    }

    #[test]
    fn u64_from_bool() {
        let nt: u64 = true._into();
        let nf: u64 = false._into();
        assert_eq!(nt, 0b1);
        assert_eq!(nf, 0);
    }

    #[test]
    fn fpte_from_raw_pwr() {
        let raw = 0b10111; // Auto extension
        let rawpte = RawPTEntry::new(raw);
        let fpte = PTEntry::from(rawpte);
        let comp = PTEntry::new(true, true, true, 0b10);
        assert_eq!(comp, fpte);
    }

    #[test]
    fn fpte_from_raw_p() {
        let raw = 0b10001; // Auto extension
        let rawpte = RawPTEntry::new(raw);
        let fpte = PTEntry::from(rawpte);
        let comp = PTEntry::new(true, false, false, 0b10);
        assert_eq!(comp, fpte);
    }

    #[test]
    fn fpte_from_raw_pr() {
        let raw = 0b10101; // Auto extension
        let rawpte = RawPTEntry::new(raw);
        let fpte = PTEntry::from(rawpte);
        let comp = PTEntry::new(true, false, true, 0b10);
        assert_eq!(comp, fpte);
    }
}
