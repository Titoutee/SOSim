use crate::mem::config::MEM_CTXT;

pub type Addr = u32; // Address are limited to 32-bit as the machine supports simulation only up to 32-bit
pub type RawAddr = u64;
pub const KERNBASE: Addr = 0; // Will probably change later on... For now kernel base is 0 

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Address {
    Virtual(Addr, usize),
    Physical(Addr, usize),
}

#[derive(Copy, Clone, Debug)]
pub struct Virtual(Address);
impl Virtual {
    pub fn new(vaddr: Addr, ptr: usize) -> Self {
        Self(Address::Virtual(vaddr, ptr))
    }

    pub fn get(&self) -> Address {
        self.0
    }

    pub fn as_phys(&self) -> Physical {
        Physical::new(self.0.translate().get_address(), self.0.get_ptr() as usize)
    }

    // Mask the virtual address to extract the page table indices for each level and the offset.
    pub fn mask(&self, lvl: u8) -> Vec<Addr> {
        let mut addr = self.0.get();
        let lmask = MEM_CTXT.lvl_mask;
        let off_mask = MEM_CTXT.off_mask;
        let off_bit_len = MEM_CTXT.v_addr_off_len;
        let lvl_bit_len = MEM_CTXT.v_addr_lvl_len;

        match lvl {
            0 => vec![addr & off_mask], // For the offset, we just mask the virtual address with the offset mask to get the offset bits.
            _ => {
                let mut levels = vec![];
                addr = addr >> off_bit_len;
                for _ in 0..MEM_CTXT.pt_levels {
                    levels.push(addr & lmask);
                    addr = addr >> lvl_bit_len;
                }
                levels
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Physical(Address);
impl Physical {
    pub fn new(paddr: u32, ptr: usize) -> Self {
        Self(Address::Physical(paddr, ptr))
    }

    pub fn get(&self) -> Address {
        self.0
    }

    // Construct a virtual address from a physical address.
    pub fn from(ppn: u32, offset: u32, ptr: usize) -> Self {
        // ppn is already shifted
        // Construct a physical address from a physical page number and an offset.
        let ppn = ppn << MEM_CTXT.v_addr_off_len;
        let paddr = ppn | offset;
        Self(Address::Physical(paddr, ptr))
    }
}

impl Address {
    // Translate a virtual address to a physical address by walking the page table hierarchy.
    pub fn translate(&self) -> Self {
        match *self {
            Self::Virtual(vaddr, ptr) => {
                // TODO: Extract page table indices using mask method
                let virt = Virtual(*self);
                let indices = virt.mask(1);
                // TODO: Walk through page table hierarchy at each level
                let mut current_addr = vaddr;
                for _ in indices {
                    // Load the page table entry at the current level
                    let pte = unsafe { *(current_addr as *const u32) }; // Placeholder for the actual memory access, which would involve reading from the page table in memory. In a real implementation, this would need to be done carefully to avoid unsafe behavior and to handle page faults, etc.
                    // Extract the physical page number and use it for the next level
                    current_addr = (pte >> MEM_CTXT.v_addr_off_len) << MEM_CTXT.v_addr_off_len;
                    // Continue to next level or extract final physical address
                }
                // Extract the offset from the virtual address
                let offset = virt.mask(0)[0];
                // Combine the physical page number with the offset to get the final physical address.
                current_addr = (current_addr & !MEM_CTXT.off_mask) | offset;
                Self::Physical(current_addr, ptr)
            }
            Self::Physical(paddr, ptr) => Self::Virtual(paddr + KERNBASE, ptr), // Nobody translates physical addresses to virtual addresses, but this is here for testing purposes, and it is a simple offset translation, as the kernel is mapped at KERNBASE in the virtual address space.
        }
    }

    pub fn get_address(&self) -> u32 {
        match *self {
            Self::Virtual(vaddr, _) => vaddr & !0xFFF,
            Self::Physical(paddr, _) => paddr & !0xFFF,
        }
    }

    pub fn get_dir_index(&self) -> usize {
        match *self {
            Self::Virtual(vaddr, _) => {
                ((vaddr >> MEM_CTXT.v_addr_off_len) & MEM_CTXT.lvl_mask) as usize
            }
            Self::Physical(_, _) => 0,
        }
    }

    pub fn get(&self) -> u32 {
        match *self {
            Self::Virtual(vaddr, _) => vaddr,
            Self::Physical(paddr, _) => paddr,
        }
    }

    pub fn get_table_index(&self) -> usize {
        match *self {
            Self::Virtual(vaddr, _) => ((vaddr) & MEM_CTXT.off_mask) as usize,
            Self::Physical(_, _) => 0,
        }
    }

    pub fn get_ptr(&self) -> *mut u32 {
        match *self {
            Self::Virtual(_, ptr) => ptr as *mut u32,
            Self::Physical(_, ptr) => ptr as *mut u32,
        }
    }

    pub fn get_offset(&self) -> u32 {
        match *self {
            Self::Virtual(vaddr, _) => vaddr & 0xFFF,
            Self::Physical(paddr, _) => paddr & 0xFFF,
        }
    }
}

#[cfg(test)]

mod tests {

    const RAW_ADDR: u32 = 0b11111111000000000011111111110000;
    use super::{Address, Physical, Virtual};

    #[cfg(feature = "bit32")]
    #[test]
    fn vaddr_from_raw_addr_32b() {
        use super::Address;

        let vaddr = dbg!(Address::Virtual(RAW_ADDR, 0));
        let offset = 0b111111110000;
        let lvl1 = 0b000000011;
        let lvl2 = 0b111111000;
        let lvl3 = 0b000001111;
        let lvl4 = 0b111100000;
        let v = vec![lvl1, lvl2, lvl3, lvl4];
    }
    #[cfg(feature = "bit32")]
    #[test]
    fn vaddr_mask_offset() {
        let vaddr = Address::Virtual(RAW_ADDR, 0);
        let virt = Virtual(vaddr);
        let offset = virt.mask(0);
        assert_eq!(offset[0], RAW_ADDR & 0xFFF);
    }

    #[cfg(feature = "bit32")]
    #[test]
    fn vaddr_mask_levels() {
        let vaddr = Address::Virtual(RAW_ADDR, 0);
        let virt = Virtual(vaddr);
        let levels = virt.mask(1);
        assert!(!levels.is_empty());
    }

    #[cfg(feature = "bit32")]
    #[test]
    fn physical_construction() {
        let ppn = 0x12345;
        let offset = 0x678;
        let phys = Physical::from(ppn, offset, 0);
        let addr = phys.get();
        match addr {
            super::Address::Physical(paddr, _) => {
                assert_eq!(paddr & 0xFFF, offset);
            }
            _ => panic!("Expected physical address"),
        }
    }

    #[cfg(feature = "bit32")]
    #[test]
    fn address_get_offset() {
        let vaddr = Address::Virtual(0xABC, 0);
        assert_eq!(vaddr.get_offset(), 0xABC);
    }

    #[cfg(feature = "bit32")]
    #[test]
    fn virtual_to_physical_conversion() {
        let virt = Virtual::new(RAW_ADDR, 0);
        let addr = virt.get();
        match addr {
            super::Address::Virtual(va, _) => {
                assert_eq!(va, RAW_ADDR);
            }
            _ => panic!("Expected virtual address"),
        }
    }
}
