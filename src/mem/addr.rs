// General purpose thin-wrapper
pub struct Addr {
    inner: u32, // (!)
}

pub struct _VAddrRawCtxt {
    lvl_size: usize, // How many page-table entries can be accessed by one level (= numer of page-table entries)
    offset_size: usize,
    sign_ext: usize,
}

impl _VAddrRawCtxt {
    pub fn new(lvl_size: usize, offset_size: usize, sign_ext: usize) -> Self {
        Self {
            lvl_size,
            offset_size,
            sign_ext,
        }
    }
    // pub fn from_offset()
}

// V-addresses are split into PT level fields, some of which may be disabled for a given setup/arch,
// and an offset for in-page inedexing.

pub struct VAddr {
    lvl1: Option<u16>,
    lvl2: Option<u16>,
    lvl3: Option<u16>,
    lvl4: Option<u16>,
    offset: Option<u16>,
}

impl VAddr {
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
    // TODO: adapt to all bitsizes
    pub fn from_raw_addr(_raw: u64) -> Self {
        let lvl_mask = 0b111111111;
        let off_mask = 0b111111111111;

        let offset = Some((_raw & off_mask) as u16);
        let lvl1 = Some((_raw >> (9) & lvl_mask) as u16);
        let lvl2 = Some((_raw >> (2 * 9) & lvl_mask) as u16);
        let lvl3 = Some((_raw >> (3 * 9) & lvl_mask) as u16);
        let lvl4 = Some((_raw >> (4*9) & lvl_mask) as u16);

        Self {
            lvl1,
            lvl2, 
            lvl3,
            lvl4,
            offset,
        }
    }
}
