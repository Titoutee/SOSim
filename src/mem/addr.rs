use super::{BitMode, MemContext};
use std::ops::Add;

type AddrInner = u64;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
// General purpose address thin-wrapper
pub struct Addr {
    pub inner: AddrInner, // (!)
}

impl Add for Addr {
    type Output = AddrInner;
    fn add(self, rhs: Self) -> Self::Output {
        self.inner + rhs.inner // "+" yields a raw u64 address, without the thin wrapper
    }
}

impl From<u64> for Addr {
    fn from(value: u64) -> Self {
        Self { inner: value }
    }
}

impl Into<u64> for Addr {
    fn into(self) -> u64 {
        self.inner
    }
}

impl Addr {
    pub fn new(raw: u64) -> Self {
        Addr { inner: raw }
    }
}

// mere testing
pub fn _mask(mut raw: u64, ctxt: &MemContext, idx: u8) -> u64 {
    assert!(idx < ctxt.pt_levels);

    let lmask = ctxt.lvl_mask;
    let omask = ctxt.off_mask;
    let off_bit_len = ctxt.v_addr_off_len;
    let lvl_bit_len = ctxt.v_addr_lvl_len;

    match idx {
        0 => raw & omask,
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

    // Context-wise
    // (!)
    /// Creates a VAddr from the raw field-cut format bit-set, in a manner depending on the running bitmode
    pub fn from_addr(_raw: u64 /*(!)*/, memctxt: &MemContext) -> Self {
        // Sign extension is ignored and does not produce any error if incorrect (aka if not copy of MSB)

        let offset = (_raw & (memctxt.off_mask as u64)) as u16;
        let mut lvls = [None; 4];
        let mut lvls_bin = _raw >> memctxt.v_addr_off_len;

        for i in 0..(memctxt.pt_levels) as usize {
            lvls[i] = Some((lvls_bin & memctxt.lvl_mask as u64) as u16);
            lvls_bin = lvls_bin >> (memctxt.v_addr_lvl_len as usize);
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

    // pub fn lvl1(&self) -> Option<u16> {
    //     self.lvl1
    // }
    //
    // pub fn lvl2(&self) -> Option<u16> {
    //     self.lvl2
    // }
    //
    // pub fn lvl3(&self) -> Option<u16> {
    //     self.lvl3
    // }
    //
    // pub fn lvl4(&self) -> Option<u16> {
    //     self.lvl4
    // }

    pub fn to_addr(&self, memctxt: &MemContext) -> Addr {
        todo!()
    }
}

#[cfg(test)]

mod tests {
    use crate::mem::MemContext;

    const raw_addr: u64 = 0b0000000000000000111100000000001111111111000000000011111111110000;

    #[cfg(feature = "bit64")]
    #[test]
    fn vaddr_from_raw_addr_64b() {
        use crate::mem::addr::_mask;

        use super::VirtualAddress;

        let ctxt = MemContext::new();
        let vaddr = dbg!(VirtualAddress::from_addr(raw_addr, &ctxt));
        let offset = 0b111111110000;
        let lvl1 = 0b000000011;
        let lvl2 = 0b111111000;
        let lvl3 = 0b000001111;
        let lvl4 = 0b111100000;
        // _mask(raw_addr, &ctxt, 4);
        assert_eq!(offset, vaddr.offset);
        assert_eq!(lvl1, vaddr.lvl1.unwrap());
        assert_eq!(lvl2, vaddr.lvl2.unwrap());
        assert_eq!(lvl3, vaddr.lvl3.unwrap());
        assert_eq!(lvl4, vaddr.lvl4.unwrap());

        // todo!()
    }
}
