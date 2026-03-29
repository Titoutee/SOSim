//! Process behaviour

use crate::lang::Command::{self, *};
use crate::mem::MemResult;
use crate::mem::Memory;
use std::sync::Arc;

// TODO: Replace with the correct module path if PageTable is defined elsewhere
#[derive(Debug, Default)]
pub struct PageTable;

#[derive(Debug, Default)]
pub struct ProcessContext {
    pub registers: [u32; 32], // General-purpose registers
    pub pc: u32,              // Program counter
}

// Signal to send to the client in responding to requests.
// Manual discriminants correspond to the signalling specification detailed in readme.
// This signal specification is some sort of enum serialization.

// THIS EXACT spec has to be used as part of any client that pays attention to the server payloads:
pub enum Signal {
    Debug = 5,
    Alloc = 1,
    Dealloc = 2,
    Write = 3,
    Read = 4,
    Exit = 0,
}

/// A single `Process` instantiated into main memory. It has its own `PageTable` and process context.
pub struct Process {
    pub pid: usize,
    pub mem: Arc<Memory>, // Backup reference to main memory
    pub page_table: PageTable,
    pub context: ProcessContext,
}

impl Process {
    /// A new process can only be created through the machine.

    /// Executes an executable command (that is every command but `EXIT`).
    /// The `EXIT` case is handled externally as part of the toplevel behaviour, as this is a toplevel-only command.
    pub fn _exec(&mut self, command: &Command) -> MemResult<Signal> {
        match command {
            Debug => {
                // println!("Debug!");
                println!("{}", self.mem);
                Ok(Signal::Debug)
            }
            Alloc(s) => Ok(Signal::Alloc),
            Dealloc(s) => {
                println!("Dealloc: {:?}", s);
                Ok(Signal::Dealloc)
            }
            Write(s) => {
                println!("Write: {:?}", s);
                Ok(Signal::Write)
            }
            Read(s) => {
                println!("Read: {:?}", s);
                Ok(Signal::Read)
            }
            Exit => {
                println!("Exit");
                Ok(Signal::Exit)
            }

            _ => unimplemented!(),
        }
    }
}
