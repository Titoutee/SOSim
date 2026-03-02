//! Process behaviour

use crate::lang::Command::{self, *};
use crate::mem::Memory;

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
pub struct Process<'a> {
    pub pid: usize,
    pub mem: &'a Memory, // Back up reference to main mem
    pub page_table: PageTable,
    pub context: ProcessContext,
}

impl<'a> Process<'a> {
    /// A new process can only be created through the machine.

    /// Executes an executable command (that is every command but `EXIT`).
    /// The `EXIT` case is handled externally as part of the
    pub fn _exec(command: &Command) -> Signal {
        match command {
            Debug => {
                println!("Debug!");
                Signal::Debug
            }
            Alloc(s) => {
                println!("Alloc: {:?}", s);
                Signal::Alloc
            }
            Dealloc(s) => {
                println!("Dealloc: {:?}", s);
                Signal::Dealloc
            }
            Write(s) => {
                println!("Write: {:?}", s);
                Signal::Write
            }
            Read(s) => {
                println!("Read: {:?}", s);
                Signal::Read
            }
            Exit => {
                println!("Exit");
                Signal::Exit
            }

            _ => unimplemented!(),
        }
    }
}
