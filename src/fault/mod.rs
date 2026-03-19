//! Faults and exceptions.

use crate::mem::addr::Addr;

#[derive(Debug)]
pub struct Fault {
    _type: FaultType,
}

impl Fault {
    pub fn _from(typ: FaultType) -> Self {
        Self { _type: typ }
    }
}

#[derive(Debug)]
pub enum FaultType {
    BufferOverflow(Addr),
    StackOverflow(Addr),
    NullPointerDeref(Addr),
    AddrOutOfRange(Addr),
    Unrecoverable,
    InvalidPage,
    ReadPermissionDenied(Addr),
    WritePermissionDenied(Addr),
    UnknownVar(Addr),
    // ...
}
