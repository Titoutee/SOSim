use crate::mem::config::MEM_CTXT;

pub type Addr = u32; // Address are limited to 32-bit as the machine supports simulation only up to 32-bit
pub type RawAddr = u64;
pub const KERNBASE: Addr = 0; // Will probably change later on... For now kernel base is 0 

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Address {
    Virtual(Addr, usize), // The usize is a pointer to the actual memory location in the simulator's memory space, which is used for reading/writing the contents of the address. This is necessary because in a real system, the virtual and physical addresses are just numbers, but in our simulator, we need to keep track of where in our simulated memory these addresses point to.
    Physical(Addr, usize), // The usize is a pointer to the actual memory location in the simulator's memory space, which is used for reading/writing the contents of the address. This is necessary because in a real system, the virtual and physical addresses are just numbers, but in our simulator, we need to keep track of where in our simulated memory these addresses point to.
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

    // Mask the virtual address to extract the page table indices for each level and the offset
    pub fn mask(&self, lvl: u8) -> Vec<Addr> {
        let mut addr = self.0.get();
        let lmask = MEM_CTXT.lvl_mask;
        let off_mask = MEM_CTXT.off_mask;
        let off_bit_len = MEM_CTXT.v_addr_off_len;
        let lvl_bit_len = MEM_CTXT.v_addr_lvl_len;

        let mut indices = Vec::new();
        if lvl == 0 {
            indices.push(addr & off_mask); // Extract the offset using the offset mask
        } else {
            for _ in 0..lvl {
                let index = (addr >> off_bit_len) & lmask; // Extract the page table index for the current level using the level mask
                indices.push(index);
                addr >>= lvl_bit_len; // Shift the address to the right by the number of bits used for the page table index to prepare for the next level
            }
        }
        indices
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

mod tests_addr {

    const RAW_ADDR: u32 = 0b111111000000000011111111110000;
    use super::{Address, Physical, Virtual};

    #[cfg(feature = "bit32")]
    #[test]
    fn vaddr_from_raw_addr_32b() {
        use super::Address;

        let virt = dbg!(Virtual(Address::Virtual(RAW_ADDR, 0)));
        let offset = 0b111111110000;
        let lvl1 = 0b000000011;
        let lvl2 = 0b111111000;
        // let lvl3 = 0b000001111;
        // let lvl4 = 0b111100000;
        let v = vec![lvl1, lvl2];
        assert_eq!(virt.mask(0)[0], offset);
        assert_eq!(virt.mask(2), v);
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

    #[test]
    fn physical_addr_get() {
        let phys = Physical::new(0x1234, 0);
        let addr = phys.get().get();
        assert_eq!(addr, 0x1234);
    }
}
