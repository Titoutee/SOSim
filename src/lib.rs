// Every bit marked with (!) means that more versatility (bitmode, architecture variations) or details will
// be given in the future to this piece of functionality.
// Alternatively, it can pinpoint unecessary implemenatation bits or method/function/procedure calls (mostly cloning, ...).

use mem::MemContext;
use process::Process;

use crate::mem::MMU;

pub mod allocator;
pub mod ext;
pub mod lang;
pub mod fault;
pub mod mem;
pub mod paging;
pub mod process;

#[allow(unused)]
pub struct Machine<'a> {
    ctxt: MemContext,
    ram: MMU<'a>,
    processes: Vec<Process>,
}
