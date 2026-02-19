use crate::mem::config::MEM_CTXT;

pub type Addr = u32; // Address are limited to 32-bit as the machine supports simulation only up to 32-bit
pub type RawAddr = u64;
pub const KERNBASE: Addr = 0;

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

    pub fn mask(&self, lel: u8) -> Vec<Addr> {
        let mut addr = self.0.get();
        let lmask = MEM_CTXT.lvl_mask;
        let off_mask = MEM_CTXT.off_mask;
        let off_bit_len = MEM_CTXT.v_addr_off_len;
        let lvl_bit_len = MEM_CTXT.v_addr_lvl_len;

        match lel {
            0 => vec![addr & off_mask],
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

    pub fn from(ppn: u32, offset: u32, ptr: usize) -> Self {
        let paddr = ppn | offset;
        Self(Address::Physical(paddr, ptr))
    }
}

impl Address {
    pub fn translate(&self) -> Self {
        match *self {
            Self::Virtual(vaddr, ptr) => Self::Physical(vaddr - KERNBASE, ptr),
            Self::Physical(paddr, ptr) => Self::Virtual(paddr + KERNBASE, ptr),
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
}
