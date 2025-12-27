// PTE format does not exactly match the x86_64 standard, as only present, write, read bits and the address payload is are serialized
// into the 64b bitset.

use super::addr::{Addr, VirtualAddress};
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
    // pub fn empty() -> Self {
    //     PageTable { arr: vec![] }
    // }

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

    pub fn translation(&self, vaddr: VirtualAddress) {}
}

/// Multilevel page table, used by default by `32b` config.
pub struct PageDirectory {
    levels: [Option<PageTable>; 2],
}

// Used for pretending to be x86
/// A RawPTEntry is really just a bitset encoding the specified bits and payloads.
// pub struct RawPTEntry {
//     bitset: u64, // std bitset for PTE format in x86_64 is 64 bit long
// }
//
// impl RawPTEntry {
//     pub fn new(bitset: u64) -> Self {
//         Self { bitset }
//     }
// }

// #[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
// pub struct PTEntry {
//     present: bool,
//     write: bool,
//     read: bool,
//     phys_frame_addr: u64,
// }
//
// impl PTEntry {
//     pub fn new(present: bool, write: bool, read: bool, phys_frame_addr: u64) -> Self {
//         Self {
//             present,
//             write,
//             read,
//             phys_frame_addr,
//         }
//     }
// }
//
// // RawPTE <-> FullPTE
// impl From<RawPTEntry> for PTEntry {
//     fn from(r: RawPTEntry) -> Self {
//         let bits = r.bitset;
//
//         let present = <bool as _From<u64>>::_from(bits & 0b1);
//         let write = <bool as _From<u64>>::_from((bits >> 1) & 0b1);
//         let read = <bool as _From<u64>>::_from((bits >> 2) & 0b1);
//         let phys_frame_addr = (bits >> 3) & _PTE_PHYS_ADDR_FR_MASK;
//
//         Self {
//             present,
//             write,
//             read,
//             phys_frame_addr,
//         }
//     }
// }

#[derive(Copy, Clone)]
pub enum Flag {
    Present,
    Writable,
    Accessed,
    Dirty,
}

pub const PTXSHIFT: usize = 12;

#[derive(Clone, Copy)]
pub struct PageTableEntry(u32);

impl PageTableEntry {
    pub(crate) fn new(ppn: u32) -> Self {
        Self(ppn << PTXSHIFT & !0xFFF)
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Present => self.0 & 1 == 1,
            Flag::Writable => (self.0 >> 1) & 1 == 1,
            Flag::Accessed => (self.0 >> 5) & 1 == 1,
            Flag::Dirty => (self.0 >> 6) & 1 == 1,
        }
    }

    pub fn set_flag(&mut self, flag: Flag) {
        match flag {
            Flag::Present => self.0 |= 1,
            Flag::Writable => self.0 |= 1 << 1,
            Flag::Accessed => self.0 |= 1 << 5,
            Flag::Dirty => self.0 |= 1 << 6,
        }
    }

    pub fn clear_flag(&mut self, flag: Flag) {
        match flag {
            Flag::Present => self.0 &= !(1u32),
            Flag::Writable => self.0 &= !(1u32 << 1),

            Flag::Accessed => self.0 &= !(1u32 << 5),
            Flag::Dirty => self.0 &= !(1u32 << 6),
        }
    }

    pub fn get_address(&self) -> Addr {
        self.0 & !0xFFFu32
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

use std::{ops::Add, vec::Vec};

#[derive(Copy, Clone, Debug)]
pub struct Page {
    pub data: [u8; MEM_CTXT.page_size as usize],
    // ref_count: usize,
    pub addr: Addr,
}

static ZERO_PAGE: Page = Page {
    data: [0; MEM_CTXT.page_size],
    addr: 0,
};

impl Page {
    pub(crate) fn new(addr: u32) -> Self {
        Self {
            data: [0; MEM_CTXT.page_size as usize],
            // ref_count: 0,
            addr,
        }
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
        data: &[u8], /* The data slab to be written at addr */
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

    //  pub fn increment_refs(&mut self) {
    //      self.ref_count += 1
    //  }
    //
    //  pub fn decrement_refs(&mut self) {
    //      self.ref_count -= 1
    //  }

    pub fn copy(&mut self, other: &Page) {
        self.write::<[u8; MEM_CTXT.page_size]>(0, &other.data);
    }
}

struct Memory {
    free_list: Vec<Page>,
    used_list: Vec<Page>,
    // zero_page: &'static Page,
}

impl Memory {
    fn new() -> Self {
        let mut v: Vec<Page> = Vec::new();
        for ppn in 1..32 {
            v.push(Page::new(32 - ppn));
        }
        Self {
            free_list: v,
            used_list: Vec::new(),
            // zero_page: &ZERO_PAGE,
        }
    }

    fn pop_free(&mut self) -> Option<Page> {
        self.free_list.pop().map(|mut page| {
            page.zero();
            page
        })
    }

    fn push_free(&mut self, page: Page) {
        self.free_list.push(page);
    }

    fn push_used(&mut self, page: Page) {
        self.used_list.push(page);
    }

    fn remove_used(&mut self, page: &Page) -> Option<Page> {
        let mut idx = self.used_list.len();
        for i in 0..self.used_list.len() {
            if self.used_list[i].addr == page.addr {
                idx = i;
            }
        }
        if idx != self.used_list.len() {
            return Some(self.used_list.remove(idx));
        }
        return None;
    }

    fn used_peek(&mut self) -> Option<&mut Page> {
        let idx = self.used_list.len() - 1;
        self.used_list.get_mut(idx)
    }
}

// pub fn kinit() {
//     unsafe { MEM = Memory::new() }
// }

// pub fn kalloc<const SZ: usize>() -> Option<&'static mut Page<SZ>> {
//     unsafe {
//         match MEM.pop_free() {
//             Some(mut page) => {
//                 // page.increment_refs();
//                 MEM.push_used(page);
//                 MEM.used_peek()
//             }
//             None => None,
//         }
//     }
// }

// pub fn kfree<const SZ: usize>(page: &Page<SZ>) {
//     if page.addr == 0 {
//         return;
//     }
//     unsafe {
//         MEM.push_free(MEM.remove_used(page).unwrap());
//     }
// }
