use super::{BitMode, MemContext};

// General purpose address thin-wrapper
pub struct Addr {
    inner: u32, // (!)
}

// V-addresses are split into page table level fields, some of which may be disabled for a given setup/arch,
// and an offset for in-page inedexing.

pub struct VirtualAddress {
    lvl1: Option<u16>,
    lvl2: Option<u16>,
    lvl3: Option<u16>,
    lvl4: Option<u16>,
    offset: Option<u16>,
}

impl VirtualAddress {
    // Manual creation of V-addresses, outside any bit-context
    pub fn new(
        lvl1: Option<u16>,
        lvl2: Option<u16>,
        lvl3: Option<u16>,
        lvl4: Option<u16>,
        page_offset: Option<u16>,
    ) -> Self {
        Self {
            lvl1,
            lvl2,
            lvl3,
            lvl4,
            offset: page_offset,
        }
    }

    // Context-wise
    // (!)
    /// Creates a VAddr from the raw field-cut format bit-set, in a manner depending on the running bitmode
    pub fn from_raw_addr(_raw: u64 /*(!)*/, memctxt: MemContext) -> Self {
        // Sign extension is ignored and does not produce any error if incorrect (aka if not copy of MSB)
        // let lvl_mask = 0b111111111; // (!)
        // let off_mask = 0b111111111111; // (!)

        let offset = Some((_raw & (memctxt.off_mask as u64)) as u16);
        let mut lvls = [None; 4];

        for i in 0..(memctxt.pt_levels) as usize {
            lvls[i] = Some((_raw >> ((i + 1) * 9) & memctxt.lvl_mask as u64) as u16);
        }

        Self {
            lvl1: lvls[0],
            lvl2: lvls[1],
            lvl3: lvls[2],
            lvl4: lvls[3]  ,          
            offset,
            //
        }
    }
}

#[cfg(test)]

mod tests {
    use crate::mem::MemContext;

    const ctxt: MemContext = MemContext::from_bit_mode__compiled();
    const raw_addr: u64 = 0b0000000000111111111100000000001111111111000000000011111111110000;

    #[test]
    fn vaddr_from_raw_addr() {
        //let ctxt_64b = MemContext::new(bitmode, lvl_mask, off_mask, page_size, page_count, pt_levels, v_addr_lvl_len, v_addr_off_len, phys_bitw);
    }
}