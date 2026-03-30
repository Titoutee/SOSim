// Every bit marked with (!) means that more versatility (bitmode, architecture variations) or details will
// be given in the future to this piece of functionality.
// Alternatively, it can pinpoint unecessary implemenatation bits or method/function/procedure calls (mostly cloning, ...).

use crate::mem::MEMORY;
use process::Process;

pub mod ext;
pub mod fault;
pub mod lang;
pub mod mem;
pub mod process;

pub type ProcessList = Vec<Process>;

pub struct Machine {
    id_c: usize,
    processes: ProcessList,
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            id_c: 0,
            processes: vec![],
        }
    }

    // O(n) which is reasonable for average usecase
    pub fn get_process(&mut self, id: usize) -> Option<&mut Process> {
        self.processes.iter_mut().filter(|x| x.pid == id).next()
    }

    pub fn create_process(&mut self) -> usize {
        let p = Process {
            pid: self.id_c,
            mem: MEMORY.clone(),
            context: Default::default(),
        };
        let pid = p.pid;
        self.processes.push(p);
        self.id_c += 1;
        pid
    }
    pub fn kill_process(&mut self, id: usize) -> Option<()> {
        let _ = self.processes.get(id)?;
        self.processes.remove(id);
        Some(())
    }
}
