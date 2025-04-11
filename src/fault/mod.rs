//! Faults and excpetions.

use crate::mem::addr::Addr;

pub struct Fault {
    _type: FaultType,
}

pub enum FaultType {
    BufferOverflow(BufferError)
    // ...
}

pub struct BufferError {
    at_addr: Addr,
}