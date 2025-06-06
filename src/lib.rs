// Every bit marked with (!) means that more versatility (bitmode, architecture variations) or details will
// be given in the future to this piece of functionality.
// Alternatively, it can pinpoint unecessary implemenatation bits or method/function/procedure calls (mostly cloning, ...).

use mem::MemContext;
use proc::Process;

use crate::mem::Ram;

pub mod allocator;
pub mod ext;
pub mod fault;
pub mod mem;
pub mod paging;
pub mod proc;

#[allow(unused)]
pub struct Machine {
    ctxt: MemContext,
    ram: Ram,
    proc: Vec<Process>,
    //
}
