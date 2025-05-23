// Every bit marked with (!) means more versatility (bitmode, architecture variations) or details will
// be given in the future to this piece of functionality.

use mem::MemContext;
use proc::Process;

pub mod paging;
pub mod allocator;
pub mod fault;
pub mod mem;
pub mod proc;

pub struct Machine {
    ctxt: MemContext,
    proc: Vec<Process>,

    //
}
