// PTE format does not exactly match the x86_64 standard, as only present, write, read bits and the address payload is are serialized
// into the 64b bitset.

use super::addr::Addr;
pub use crate::ext::{_From, _Into};
use crate::mem::config::MEM_CTXT;

#[derive(Copy, Clone)]
pub enum Flag {
    Present,
    Writable,
    Read,
}

#[derive(Clone, Copy)]
pub struct PageTableEntry(u32);

impl PageTableEntry {
    pub(crate) fn new(addr: u32) -> Self {
        Self(addr << MEM_CTXT.lvl_mask & !0x7u32) // Mask the 3 LSBs, which are reserved for flags, and shift the address to the left to make room for the flags.
    }

    // Little-endian

    pub fn get_flag(&self, flag: Flag) -> bool {
        match flag {
            Flag::Present => self.0 & 1 == 1,
            Flag::Writable => (self.0 >> 1) & 1 == 1,
            Flag::Read => (self.0 >> 2) & 1 == 1,
        }
    }

    pub fn set_flag(&mut self, flag: Flag) {
        match flag {
            Flag::Present => self.0 |= 1,
            Flag::Writable => self.0 |= 1 << 1,
            Flag::Read => self.0 |= 1 << 2,
        }
    }

    pub fn clear_flag(&mut self, flag: Flag) {
        match flag {
            Flag::Present => self.0 &= !(1u32), // Mask the bit corresponding to the flag to 0, leaving the other bits unchanged.
            Flag::Writable => self.0 &= !(1u32 << 1),
            Flag::Read => self.0 &= !(1u32 << 2),
        }
    }

    pub fn get_ppn(&self) -> Addr {
        (self.0 & !0x7u32) as Addr // Mask the 3 LSBs, which are reserved for flags, to get the physical page number.
    }
}

/// A page containing its own vpn->ppn translation
#[derive(Copy, Clone, Debug)]
pub struct Page {
    pub data: [u8; MEM_CTXT.page_size as usize],
    pub ref_count: usize,
    pub ppn: u32,            // physical page number
    pub proc_id: Option<u8>, // None = unallocated
}

static ZERO_PAGE: Page = Page {
    // This page is used to handle null page exceptions, and is never allocated to any process.
    data: [0; MEM_CTXT.page_size as usize],
    ppn: 0,
    ref_count: 0,
    proc_id: None, // No one owns ZERO_PAGE, and it is read-only, so it can be safely shared across processes without any risk of data corruption.
};

impl Page {
    pub(crate) fn new(ppn: u32) -> Self {
        Self {
            data: [0; MEM_CTXT.page_size as usize],
            ref_count: 0,
            ppn,
            proc_id: None,
        }
    }

    pub fn increment_refs(&mut self) {
        self.ref_count += 1
    }

    pub fn decrement_refs(&mut self) {
        self.ref_count -= 1
    }

    pub fn ppn(&self) -> u32 {
        self.ppn
    }

    pub fn ppn_as_addr(&self) -> Addr {
        self.ppn * (MEM_CTXT.page_size as u32)
    }

    pub fn is_in(&self, addr: Addr) -> bool {
        (MEM_CTXT.page_size as u32 > addr - self.ppn_as_addr()) && (self.ppn_as_addr()) as i32 >= 0
    }

    // T can be a single u8 or a Struct (which is essentially a [u8, _])
    /// Reads a word contained in this page, using its base address to calculate the offset.
    pub fn read<T>(&self, addr: Addr) -> &[u8] {
        if self.ppn == 0 {
            return &[0]; // Null page exception
        }

        let page_addr = self.ppn_as_addr();

        let size = std::mem::size_of::<T>(); // 1 or more bytes...?
        let s = (addr - page_addr) as usize;
        let e = s + size;

        &self.data[s..e]
    }

    pub fn write<T>(
        &mut self,
        addr: Addr,
        data: &[u8], /* The data blob to be written at addr */
    ) {
        let page_addr = self.ppn_as_addr();

        let size = std::mem::size_of::<T>(); // 1 or more bytes...?
        let s = (addr - page_addr) as usize;
        let e = s + size;
        let mut i = 0;
        for idx in s..e {
            if i < data.len() {
                self.data[idx] = data[i];
                i += 1;
            }
        }
    }

    /// Zeroes a page.
    pub fn zero(&mut self) {
        self.write::<[u8; 4096]>(0, &[0; 4096]);
    }

    pub fn copy(&mut self, other: &Page) {
        self.write::<[u8; MEM_CTXT.page_size as usize]>(0, &other.data);
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
}
