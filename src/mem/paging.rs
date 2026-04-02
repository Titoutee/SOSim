// PTE format does not exactly match the x86_64 standard, as only present, write, read bits and the address payload is are serialized
// into the 64b bitset.

use super::addr::Addr;
pub use crate::ext::{_From, _Into};
use crate::fault::Fault;
use crate::mem::addr::Physical;
use crate::mem::config::MEM_CTXT;
use crate::mem::{MemResult, PHYS_TOTAL};
use std::collections::HashMap;
#[derive(Copy, Clone)]
pub enum Flag {
    Present,
    Writable,
    Read,
}

#[derive(Clone, Copy)]
pub struct PageTableEntry(pub u32);

impl PageTableEntry {
    #[allow(unused)]
    pub(crate) fn new(addr: u32) -> Self {
        Self(addr << 3 & !0x7u32) // Clear the 3 LSBs, which are reserved for flags, to ensure a clean initial state.
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
    pub base: Addr, // Base address of the page in physical memory
    pub ref_count: usize,
    pub ppn: u32,            // physical page number
    pub proc_id: Option<u8>, // None = unallocated
    pub pte: PageTableEntry, // Page table entry storing permission flags
}

pub const ZERO_PAGE_PPN: u32 = 0; // Physical page number for the zero page

pub static ZERO_PAGE: Page = Page {
    // This page is used to handle null page exceptions, and is never allocated to any process.
    data: [0; MEM_CTXT.page_size as usize],
    ppn: ZERO_PAGE_PPN,
    ref_count: 0,
    base: ZERO_PAGE_PPN * (MEM_CTXT.page_size as u32), // Base address of the zero page in physical memory
    proc_id: None, // No one owns ZERO_PAGE, and it is read-only, so it can be safely shared across processes without any risk of data corruption.
    pte: PageTableEntry(0),
};

// Page struct represents a physical page in memory, containing the actual data of the page, a reference count to track how many processes are using it, the physical page number (ppn) which identifies its location in physical memory, an optional process ID (proc_id) to indicate which process owns the page (if any), and a PageTableEntry (pte) that stores permission flags for the page. The Page struct provides methods for creating new pages, managing reference counts, checking if an address is within the page, and reading/writing data to/from the page.
impl Page {
    // At starting point, each page's base address is calculated from the ppn, and are ordered relatively to their ppn
    pub(crate) fn new(ppn: u32) -> Self {
        Self {
            data: [0; MEM_CTXT.page_size as usize],
            ref_count: 0,
            ppn,
            base: ppn * (MEM_CTXT.page_size as u32), // Calculate the base address of the page in physical memory based on its physical page number (ppn) and the page size defined in the memory context (MEM_CTXT).
            proc_id: None,
            pte: PageTableEntry(0), // Initially no flags set
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

    // Check if a given address falls within the page's address range, which is determined by its physical page number (ppn) and the page size defined in the memory context (MEM_CTXT). The method calculates the starting address of the page using its ppn and checks if the provided address is greater than or equal to this starting address and less than the starting address plus the page size. This is useful for determining if a particular memory access is valid for this page.
    pub fn ppn_as_addr(&self) -> Addr {
        self.ppn * (MEM_CTXT.page_size as u32) // Convert the physical page number to a physical address by multiplying it by the page size.
    }

    pub fn is_in(&self, addr: Addr) -> bool {
        (MEM_CTXT.page_size as u32 > addr - self.ppn_as_addr()) && (self.ppn_as_addr()) as i32 >= 0
    }

    /// Get the page table entry
    pub fn get_pte(&self) -> PageTableEntry {
        self.pte
    }

    pub fn set_pte(&mut self, pte: PageTableEntry) {
        self.pte = pte;
    }

    pub fn present(&self) -> bool {
        self.get_pte().get_flag(Flag::Present)
    }

    pub fn readable(&self) -> bool {
        self.get_pte().get_flag(Flag::Read)
    }

    pub fn writable(&self) -> bool {
        self.get_pte().get_flag(Flag::Writable)
    }

    // Writes a data blob to the page at the specified address. The size of the data blob is determined by the type parameter T, which can be any type that implements the Sized trait. The method calculates the offset within the page based on the provided address and writes the data blob to the page's data array starting from that offset. If the data blob is larger than the remaining space in the page, it will only write as much as fits within the page.
    pub fn write(
        &mut self,
        addr: Addr,
        data: Vec<u8>, /* The data blob to be written at addr */
    ) -> MemResult<()> {
        if !self.writable() {
            return Err(Fault::_from(
                crate::fault::FaultType::WritePermissionDenied(addr),
            ));
        }

        let page_addr = self.ppn_as_addr();

        let size = data.len(); // The size of the data blob to be written, determined by the type parameter T. This allows the method to know how many bytes to write based on the type of data being written.
        let s = (addr - page_addr) as usize;
        let e = s + size;
        let mut i = 0;
        for idx in s..e {
            if i < data.len() {
                self.data[idx] = data[i];
                i += 1;
            }
        }
        Ok(())
    }

    pub fn read(&self, addr: Addr, len: usize) -> MemResult<Vec<u8>> {
        if !self.readable() {
            return Err(Fault::_from(crate::fault::FaultType::ReadPermissionDenied(
                addr,
            )));
        }

        let page_addr = self.ppn_as_addr();
        let s = (addr - page_addr) as usize;
        let e = s + len;
        Ok(self.data[s..e].to_vec())
    }

    /// Zeroes a page.
    pub fn zero(&mut self) -> MemResult<()> {
        self.write(0, vec![0; 4096])
    }

    pub fn copy(&mut self, other: &Page) -> MemResult<()> {
        self.write(0, other.data.to_vec())
    }
}

// Partial
impl PartialEq for Page {
    fn eq(&self, other: &Self) -> bool {
        self.ppn == other.ppn
    }
}

impl PartialOrd for Page {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ppn.partial_cmp(&other.ppn)
    }
}

// Non partial
impl Eq for Page {}

impl Ord for Page {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ppn.cmp(&other.ppn)
    }
}

#[derive(Debug)]
pub struct FrameAllocator {
    pub free_list: HashMap<u32, Page>, // Physical frames; each frame is taken whenever a process allocates it for itself
    pub used_list: HashMap<u32, Page>, // Pages that have been allocated to processes; used for tracking and deallocation
}

// The FrameAllocator struct manages the allocation and deallocation of physical memory pages. It maintains a free list of available pages and a used list of allocated pages, along with counters for tracking the number of pages in each state. The free_list is a vector of Page structs that represent physical frames of memory that are available for allocation, while the used_list is a vector of Page structs that have been allocated to processes. The FrameAllocator provides methods for initializing the free list with a range of physical addresses, allocating pages to processes, and freeing pages back to the free list when they are no longer needed.
// Mutex is used to ensure thread safety when multiple threads may be accessing or modifying the free_list and used_list concurrently, preventing race conditions and ensuring that the internal state of the FrameAllocator remains consistent.
impl FrameAllocator {
    fn empty() -> Self {
        FrameAllocator {
            free_list: HashMap::with_capacity(MEM_CTXT.page_count as usize),
            used_list: HashMap::with_capacity(MEM_CTXT.page_count as usize),
        }
    }

    pub fn new() -> Self {
        let mut a = Self::empty();
        a.init_range(Physical::new(0, 0), Physical::new(PHYS_TOTAL as u32, 0));
        a
    }

    // Core method to initialize the free list with a range of physical addresses. It takes a starting physical address and an ending physical address, and populates the free_list with Page structs representing each page in that range. The method calculates the number of pages that can fit within the specified range based on the page size defined in the memory context (MEM_CTXT) and creates a Page struct for each page, setting its physical page number (ppn) accordingly. This allows the FrameAllocator to manage the available physical memory and allocate pages to processes as needed.
    fn init_range(&mut self, start: Physical, end: Physical) {
        let start = start.get();
        let end = end.get().get_address() as usize;
        let free = &mut self.free_list;
        free.clear();
        let mut addr = start.get_address() as usize;
        while addr + MEM_CTXT.page_size <= end {
            let ppn = (addr as u32) / (MEM_CTXT.page_size as u32);
            free.insert(ppn, Page::new(ppn));
            addr += MEM_CTXT.page_size;
        }
    }

    // Get the index of the page with the given ppn in the free list, if it exists. This is used for allocation, to find a free page in the free list and move it to the used list.
    pub fn get_free_mut(&mut self, ppn: u32) -> Option<&mut Page> {
        self.free_list.get_mut(&ppn)
    }

    // Get the index of the page with the given ppn in the used list, if it exists. This is used for deallocation, to find the page in the used list and move it back to the free list.
    pub fn get_used_mut(&mut self, ppn: u32) -> Option<&mut Page> {
        self.used_list.get_mut(&ppn)
    }

    pub fn push_free(&mut self, page: Page) {
        self.free_list.insert(page.ppn, page);
    }

    pub fn push_used(&mut self, page: Page) {
        self.used_list.insert(page.ppn, page);
    }

    pub fn free_frames(&self) -> usize {
        self.free_list.len()
    }

    pub fn used_frames(&self) -> usize {
        self.used_list.len()
    }

    pub fn total_frames(&self) -> usize {
        self.free_frames() + self.used_frames()
    }
}

#[cfg(test)]
mod tests_paging {
    use super::*;

    #[test]
    fn test_pte_new() {
        let pte = PageTableEntry::new(0x1000); // New PTE, flags clear
        assert_eq!(pte.0, 0x8000);
    }

    #[test]
    fn test_pte_set_and_get_flags() {
        let mut pte = PageTableEntry::new(0x1000); // New PTE, flags clear

        pte.set_flag(Flag::Present);
        assert!(pte.get_flag(Flag::Present));

        pte.set_flag(Flag::Writable);
        assert!(pte.get_flag(Flag::Writable));

        pte.set_flag(Flag::Read);
        assert!(pte.get_flag(Flag::Read));
    }

    #[test]
    fn test_pte_clear_flags() {
        let mut pte = PageTableEntry::new(0x1000);
        pte.set_flag(Flag::Present);
        pte.set_flag(Flag::Writable);
        pte.clear_flag(Flag::Present);

        assert!(!pte.get_flag(Flag::Present));
        assert!(pte.get_flag(Flag::Writable));
    }

    #[test]
    fn test_pte_get_ppn() {
        let pte = PageTableEntry::new(0x5000);
        assert_eq!(pte.get_ppn(), 0x28000);
    }

    #[test]
    fn test_page_new() {
        let page = Page::new(42);
        assert_eq!(page.ppn, 42);
        assert_eq!(page.ref_count, 0);
        assert!(page.proc_id.is_none());
    }

    #[test]
    fn test_page_ref_counting() {
        let mut page = Page::new(1);
        page.increment_refs();
        page.increment_refs();
        assert_eq!(page.ref_count, 2);
        page.decrement_refs();
        assert_eq!(page.ref_count, 1);
    }

    #[test]
    fn test_page_is_in() {
        let page = Page::new(1); // PPN 1 corresponds to address range 0x1000 to 0x1FFF
        assert!(page.is_in(0x1000)); // Start of page
        assert!(page.is_in(0x1FFF)); // End of page
        assert!(!page.is_in(0x2000)); // Just outside page
    }

    #[test]
    #[should_panic]
    fn test_page_write_and_read_fail_null_page() {
        let mut page = Page::new(0);
        let data = [1, 2, 3, 4].to_vec();
        page.write(0, data.clone()).unwrap();
        // Dead zone
        let read_data = page.read(0, 4).unwrap();
        assert_eq!(data.to_vec(), read_data);
    }

    #[test]
    fn test_page_write_and_read() {
        let mut page = Page::new(1);
        page.pte.set_flag(Flag::Writable);
        page.pte.set_flag(Flag::Read);
        let data = [1, 2, 3, 4].to_vec();
        page.write(4096, data.clone()).unwrap();
        let read_data = page.read(4096, 4).unwrap();
        assert_eq!(data, read_data);
    }

    #[test]
    fn test_init_frame_allocator() {
        let allocator = FrameAllocator::new();
        assert_eq!(
            allocator.free_frames(),
            PHYS_TOTAL as usize / MEM_CTXT.page_size as usize
        );
        assert_eq!(allocator.used_frames(), 0);
        assert_eq!(allocator.free_list.get(&0), Some(&Page::new(0)));
        assert_eq!(allocator.used_list.get(&0), None);
    }

    #[test]
    fn test_frame_allocator_push_free_and_used() {
        let mut allocator = FrameAllocator::new();
        let page = Page::new(0);
        allocator.push_used(page);
        assert_eq!(allocator.used_frames(), 1);
        assert_eq!(allocator.used_list.get(&0), Some(&Page::new(0)));
        allocator.push_free(page);
        assert_eq!(allocator.free_frames(), 512); // Assuming 512 pages total
        assert_eq!(allocator.free_list.get(&0), Some(&Page::new(0)));
    }
}
