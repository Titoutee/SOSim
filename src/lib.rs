// Every bit marked with (!) means that more versatility (bitmode, architecture variations) or details will
// be given in the future to this piece of functionality.
// Alternatively, it can pinpoint unecessary implemenatation bits or method/function/procedure calls (mostly cloning, ...).

use mem::MemContext;
use process::Process;

use crate::mem::{MMU, Memory};

pub mod ext;
pub mod fault;
pub mod lang;
pub mod mem;
pub mod process;

#[allow(unused)]
pub struct Machine<'a> {
    ram: Memory<'a>,
    processes: Vec<Process>,
}
