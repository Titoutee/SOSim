use crate::mem::config::MEM_CTXT;

use super::MemContext;

pub type Addr = u32; // Address are limited to 32-bit as the machine supports simulation only up to 32-bit
pub type RawAddr = u64;

// mere testing of masks
pub fn _mask(mut raw: Addr, ctxt: &MemContext, idx: u8) -> Addr {
    assert!(idx < ctxt.pt_levels);

    let lmask = ctxt.lvl_mask;
    let off_mask = ctxt.off_mask;
    let off_bit_len = ctxt.v_addr_off_len;
    let lvl_bit_len = ctxt.v_addr_lvl_len;

    match idx {
        0 => raw & off_mask,
        _ => {
            raw = raw >> off_bit_len;
            for _ in 1..ctxt.pt_levels {
                raw = raw >> lvl_bit_len;
            }
            raw & lmask
        }
    }
}

// V-addresses are split into page table level fields, some of which may be disabled for a given config and give away their bits to sign-extension,
// and an offset for in-page inedexing.

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct VirtualAddress {
    pub lvl1: Option<u16>,
    pub lvl2: Option<u16>,
    pub lvl3: Option<u16>,
    pub lvl4: Option<u16>,
    pub offset: u16,
}

impl VirtualAddress {
    // Manual creation of V-addresses, outside any bit-context
    pub fn new(
        lvl1: Option<u16>,
        lvl2: Option<u16>,
        lvl3: Option<u16>,
        lvl4: Option<u16>,
        page_offset: u16,
    ) -> Self {
        Self {
            lvl1,
            lvl2,
            lvl3,
            lvl4,
            offset: page_offset,
        }
    }

    /// A "phantom" VirtualAddress has its `lvl1` field set to `None`, which basically inicates a no-translation address.
    fn is_phantom(&self) -> bool {
        self.lvl1.is_none()
    }

    // Context-wise
    // (!)
    /// Creates a VAddr from the raw field-format bit-set, in a manner depending on the running bitmode
    pub fn from_addr(_raw: RawAddr) -> Self {
        // Sign extension is ignored and does not produce any error if incorrect (aka if not copy of MSB)

        let offset = (_raw & (MEM_CTXT.off_mask) as u64) as u16;
        let mut lvls = [None; 4];
        let mut lvls_bin = _raw >> MEM_CTXT.v_addr_off_len;

        for i in 0..(MEM_CTXT.pt_levels) as usize {
            lvls[i] = Some((lvls_bin & MEM_CTXT.lvl_mask as u64) as u16);
            lvls_bin = lvls_bin >> (MEM_CTXT.v_addr_lvl_len as usize);
            /* levels share the same mask */
        }

        println!("{:?}", lvls);

        Self {
            lvl1: lvls[0],
            lvl2: lvls[1],
            lvl3: lvls[2],
            lvl4: lvls[3],
            offset,
            //
        }
    }

    pub fn to_addr(&self, memctxt: &MemContext) -> Addr {
        todo!()
    }
}

#[cfg(test)]

mod tests {

    const RAW_ADDR: u64 = 0b0000000000000000111100000000001111111111000000000011111111110000;

    #[cfg(feature = "bit32")]
    #[test]
    fn vaddr_from_raw_addr_32b() {
        use super::VirtualAddress;

        let vaddr = dbg!(VirtualAddress::from_addr(RAW_ADDR));
        let offset = 0b111111110000;
        let lvl1 = 0b000000011;
        let lvl2 = 0b111111000;
        let lvl3 = 0b000001111;
        let lvl4 = 0b111100000;
        assert_eq!(offset, vaddr.offset);
        assert_eq!(lvl1, vaddr.lvl1.unwrap());
        assert_eq!(lvl2, vaddr.lvl2.unwrap());
        assert_eq!(lvl3, vaddr.lvl3.unwrap());
        assert_eq!(lvl4, vaddr.lvl4.unwrap());

        // todo!()
    }
}
