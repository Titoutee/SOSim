//! Faults and excpetions.

use crate::mem::addr::Addr;

pub struct Fault {
    _type: FaultType,
}

impl Fault {
    pub fn _from(typ: FaultType) -> Self {
        Self { _type: typ }
    }
}

pub enum FaultType {
    BufferOverflow(Addr),
    StackOverflow(Addr),
    NullPointerDeref(Addr),
    AddrOutOfRange(Addr),
    Unrecoverable,
    // ...
}
