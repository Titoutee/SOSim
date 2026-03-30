//! Faults and exceptions.

use crate::mem::addr::Addr;

#[derive(Debug, PartialEq, Eq)]
pub struct Fault {
    _type: FaultType,
}

impl Fault {
    pub fn _from(typ: FaultType) -> Self {
        Self { _type: typ }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum FaultType {
    BufferOverflow(Addr),
    StackOverflow(Addr),
    NullPointerDeref(Addr),
    AddrOutOfRange(Addr),
    InvalidPPN(u32),
    Unrecoverable,
    InvalidPage(u32),
    BadSegment,
    Occupied(Addr),
    ReadPermissionDenied(Addr),
    WritePermissionDenied(Addr),
    UnknownVar(Addr),
    SignallingFault,
    // ...
}
