//! Process behaviour

use crate::lang::Command::{self, *};
use crate::mem::Memory;
use crate::mem::paging::PageTable;

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
    pub id: usize,
    pub mem: &'a Memory, // Back up reference to main mem
    // pub ctxt: ProcContext,
    pub pt: PageTable,
    // pub v_space_free: Vec<Page<SZ>>, // Virtual pages
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
