// Every bit marked with (!) means that more versatility (bitmode, architecture variations) or details will
// be given in the future to this piece of functionality.
// Alternatively, it can pinpoint unecessary implemenatation bits or method/function/procedure calls (mostly cloning, ...).

use process::Process;

use crate::mem::{MemContext, Memory, paging::PageTable};

pub mod ext;
pub mod fault;
pub mod lang;
pub mod mem;
pub mod process;

pub type ProcessList<'a> = Vec<Process<'a>>;

pub struct Machine<'a> {
    id_c: usize,
    mem: Memory<'a>,
    processes: ProcessList<'a>,
}

impl<'a> Machine<'a> {
    pub fn new(ctxt: &'a MemContext) -> Machine<'a> {
        let mem = Memory::new(ctxt);
        Machine {
            id_c: 0,
            processes: vec![],
            mem,
        }
    }

    // O(n) which is reasonable for average usecase
    pub fn get_process(&self, id: usize) -> Option<&Process<'a>> {
        self.processes.iter().filter(|x| x.id == id).next()
    }

    pub fn create_process(&'a mut self) {
        let p = Process {
            id: self.id_c,
            mem: &self.mem,
            pt: PageTable::new_init(self.mem.context.page_count as usize),
        };
        self.processes.push(p);
        self.id_c += 1;
    }

    pub fn add_process(&'a mut self, mut new: Process<'a>) {
        new.mem = &self.mem;
        self.processes.push(new);
    }

    pub fn kill_process(&mut self, id: usize) -> Option<()> {
        let _ = self.processes.get(id)?;
        self.processes.remove(id);
        Some(())
    }

    pub fn mem_context(&self) -> &MemContext {
        self.mem.context
    }
}
