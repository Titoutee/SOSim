//! Faults and excpetions.

use crate::mem::addr::Addr;

pub struct Fault {
    _type: FaultType,
}

impl Fault {
    pub fn from(typ: FaultType) -> Self {
        Self {_type: typ}
    }
}

pub enum FaultType {
    BufferOverflow(Addr),
    NullPointerDeref(Addr),
    AddrOutOfRange(Addr),
    // ...
}