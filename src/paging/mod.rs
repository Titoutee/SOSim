// PTE format does not exactly match the x86_64 standard, as only present, write, read bits and the address payload is are serialized
// into the 64b bitset.
pub use crate::ext;
use crate::mem::config::bitmode::_PAGE_COUNT;

use super::mem::{
    addr::{Addr, VirtualAddress},
    config::bitmode::_PTE_PHYS_ADDR_FR_MASK,
};
pub use ext::{_From, _Into};

/// A pagetable consisting of a capacity-cap-ed collection of pagetable entries (see `PTEntry`).
///
/// Off entries, for unallocated pages, are `None`, but this state is not guaranteed at the initialisation time by
/// `new_init`, which simply returns a table with a capacity of `_PAGE_COUNT`.
pub struct PageTable {
    // PTEs are hard-indexed, which means the index part (9 bit for 64-bit v-addr) in the v-addr is directly used to access the appropriate PTE
    arr: Vec<Option<PTEntry>>, // /!\ !pub for API instanciation only
}

impl PageTable {
    // pub fn empty() -> Self {
    //     PageTable { arr: vec![] }
    // }

    /// Inits the pagetable with an inner vector of capacity `_PAGE_COUNT`.
    pub fn new_init() -> Self {
        PageTable {
            arr: Vec::with_capacity(_PAGE_COUNT as usize),
        }
    }

    /// Inserts a PTE into the pagetable given a level field of a virtual address *(may change in the future into providing the virtual
    /// address and handling the level disjunction within this piece of behaviour)*.
    ///
    /// If insertion due to pagetable overloading or `None` address level, `None` is returned.
    pub fn add_pte(&mut self, pte: PTEntry, at_addr: Option<u16>) -> Option<()> {
        if self.arr.len() >= self.arr.capacity() {
            return None;
        }
        self.arr.insert(at_addr? as usize, Some(pte));
        Some(())
    }

    /// Retrieves the physical frame base address for the PTE at add `at_addr`.
    /// 
    /// This is the only step within the translation process which includes interaction which the pagetable.
    pub fn _get_frame_addr(&self, at_addr: Option<u16>) -> Option<Addr> {
        Some(
            (*self.arr.get(at_addr? as usize)?)
                .clone()?
                .phys_frame_addr
                .into(),
        ) // (!)
    }
}

/// Multilevel page table, used by default by `64b` config.
pub struct PageDirectory {
    levels: [Option<PageTable>; 4],
}

// Used for pretending to be x86
pub struct RawPTEntry {
    bitset: u64, // std bitset for PTE format in x86_64 is 64 bit long
}

impl RawPTEntry {
    pub fn new(bitset: u64) -> Self {
        Self { bitset }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
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
