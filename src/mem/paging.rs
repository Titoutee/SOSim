// PTE format does not exactly match the x86_64 standard, as only present, write, read bits and the address payload is are serialized
// into the 64b bitset.

use super::addr::{Addr, Address};
pub use crate::ext::{_From, _Into};
use crate::mem::config::MEM_CTXT;

/// A pagetable consisting of a capacity-cap-ed collection of pagetable entries (see `PTEntry`).
///
/// Off entries, for unallocated pages, are `None`, but this state is not guaranteed at the initialisation time by
/// `new_init`, which simply returns a table with a capacity of `_PAGE_COUNT`.
pub struct PageTable {
    // PTEs are hard-indexed, which means the index part (9 bit for 64-bit v-addr) in the v-addr is directly used to access the appropriate PTE
    arr: Vec<Option<PageTableEntry>>, // /!\ pub for API instanciation only
}

impl PageTable {
    /// Initialises the pagetable with an inner vector of capacity `_PAGE_COUNT`.
    /// This capacity should be kept untouched, as specified by the config's page table length.
    pub fn new_init(page_count: usize) -> Self {
        PageTable {
            arr: Vec::with_capacity(page_count),
        }
    }

    /// Inserts a PTE into the pagetable given a level field of a virtual address *(may change in the future into providing the virtual
    /// address and handling the level disjunction within this piece of behaviour)*.
    ///
    /// If insertion due to pagetable overloading or `None` address level, `None` is returned.
    pub fn add_pte(&mut self, pte: PageTableEntry, at_addr: Option<u16>) -> Option<()> {
        if self.arr.len() >= self.arr.capacity() {
            return None;
        }
        self.arr.insert(at_addr? as usize, Some(pte));
        Some(())
    }

    /// Retrieves the physical frame address for the PTE at address `at_addr`.
    ///
    /// This is the only step within the translation process which includes interaction with the page table.
    pub fn _get_frame_addr(&self, at_addr: Option<u16>) -> Option<Addr> {
        Some(
            (*self.arr.get(at_addr? as usize)?)
                .clone()?
                .get_address()
                .into(),
        ) // (!)
    }

    pub fn translation(&self, vaddr: Address) {}
}

/// Multilevel page table, used by default by `32b` config.
pub struct PageDirectory {
    levels: [Option<PageTable>; MEM_CTXT.pt_levels as usize],
}

#[derive(Copy, Clone)]
pub enum Flag {
    Present,
    Writable,
}

#[derive(Clone, Copy)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    pub(crate) fn new(addr: u32) -> Self {
        Self((addr as u64) << MEM_CTXT.lvl_mask & !0xFFFu64)
    }

    // Little-endian

    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Present => self.0 & 1 == 1,
            Flag::Writable => (self.0 >> 1) & 1 == 1,
        }
    }

    pub fn set_flag(&mut self, flag: Flag) {
        match flag {
            Flag::Present => self.0 |= 1,
            Flag::Writable => self.0 |= 1 << 1,
        }
    }

    pub fn clear_flag(&mut self, flag: Flag) {
        match flag {
            Flag::Present => self.0 &= !(1u64), // Mask the entire entry, zero-ing only LSB
            Flag::Writable => self.0 &= !(1u64 << 1),
        }
    }

    pub fn get_address(&self) -> Addr {
        (self.0 & !0xFFFu64) as Addr
    }
}

#[cfg(test)]
mod tests {
    use super::{_From, _Into};

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

    // #[test]
    // fn fpte_from_raw_pwr() {
    //     let raw = 0b10111; // Auto extension
    //     let rawpte = RawPTEntry::new(raw);
    //     let fpte = PTEntry::from(rawpte);
    //     let comp = PTEntry::new(true, true, true, 0b10);
    //     assert_eq!(comp, fpte);
    // }

    // #[test]
    // fn fpte_from_raw_p() {
    //     let raw = 0b10001; // Auto extension
    //     let rawpte = RawPTEntry::new(raw);
    //     let fpte = PTEntry::from(rawpte);
    //     let comp = PTEntry::new(true, false, false, 0b10);
    //     assert_eq!(comp, fpte);
    // }

    // #[test]
    // fn fpte_from_raw_pr() {
    //     let raw = 0b10101; // Auto extension
    //     let rawpte = RawPTEntry::new(raw);
    //     let fpte = PTEntry::from(rawpte);
    //     let comp = PTEntry::new(true, false, true, 0b10);
    //     assert_eq!(comp, fpte);
    // }
}

use std::vec::Vec;

#[derive(Copy, Clone, Debug)]
pub struct Page {
    pub data: [u8; MEM_CTXT.page_size as usize],
    // ref_count: usize,
    pub addr: Addr,
    pub proc_id: Option<u8>, // None = unallocated
}

static ZERO_PAGE: Page = Page {
    data: [0; MEM_CTXT.page_size as usize],
    addr: 0,
    proc_id: None, // No one owns ZERO.
};

impl Page {
    pub(crate) fn new(addr: Addr, id: u8) -> Self {
        Self {
            data: [0; MEM_CTXT.page_size as usize],
            // ref_count: 0,
            addr,
            proc_id: Some(id),
        }
    }

    pub fn is_in(&self, addr: Addr) -> bool {
        (MEM_CTXT.page_size as u32 > addr - self.addr) && (addr - self.addr) as i32 >= 0
    }

    // T can be a single u8 or a Struct (which is essentially a [u8, _])
    /// Reads a word contained in this page, using its base address to calculate the offset.
    pub fn read<T>(&self, addr: Addr) -> &[u8] {
        if self.addr == 0 {
            return &[0]; // Null page exception
        }

        let size = std::mem::size_of::<T>(); // 1 or more bytes...?
        let s = (addr - self.addr) as usize;
        let e = s + size;

        &self.data[s..e]
    }

    pub fn write<T>(
        &mut self,
        addr: Addr,
        data: &[u8], /* The data blob to be written at addr */
    ) {
        let size = std::mem::size_of::<T>(); // 1 or more bytes...?
        let s = (addr - self.addr) as usize;
        let e = s + size;
        let mut i = 0;
        for idx in s..e {
            if i < data.len() {
                self.data[idx] = data[i];
                i += 1;
            }
        }
    }

    pub fn addr(&self) -> Addr {
        self.addr
    }

    /// Zeroes a page.
    fn zero(&mut self) {
        self.write::<[u8; 4096]>(0, &[0; 4096]);
    }

    pub fn copy(&mut self, other: &Page) {
        self.write::<[u8; MEM_CTXT.page_size as usize]>(0, &other.data);
    }
}
