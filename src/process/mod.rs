//! Process behaviour

use crate::lang::Byte;
use crate::lang::Command::{self, *};
use crate::lang::parse::{_AllocReq, _AllocStructReq, _DeallocReq, _ReadReq, _WriteReq};
use crate::mem::MemResult;
use crate::mem::Memory;
use std::sync::{Arc, Mutex};
// TODO: Replace with the correct module path if PageTable is defined elsewhere

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
    Empty = 6,
    Debug = 5,
    Alloc = 1,
    Dealloc = 2,
    Write = 3,
    Read = 4,
    Exit = 0,
    Fault = 7,
}

/// A single `Process` instantiated into main memory. It has its own `PageTable` and process context.
pub struct Process {
    pub pid: usize,
    pub mem: Arc<Mutex<Memory>>, // Backup reference to main memory
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
                println!("{}", self.mem.lock().unwrap());
                Ok(Signal::Debug)
            }
            // Allocs and writes
            Alloc(s) => {
                let addr = s.at.expect("Expected an address inside the request");
                self.mem.lock().unwrap()._alloc(addr, 1)?;

                self.mem.lock().unwrap()._write_at_addr(addr, vec![0])?;
                Ok(Signal::Alloc)
            }
            AllocStruct(s) => {
                let addr = s.at.expect("Expected an address inside the request");
                self.mem.lock().unwrap()._alloc(addr, s.fields.len())?;
                self.mem
                    .lock()
                    .unwrap()
                    ._write_at_addr(addr, vec![0; s.fields.len()])?;
                Ok(Signal::Alloc)
            }
            Dealloc(s) => {
                let addr = s.at;
                self.mem.lock().unwrap()._dealloc(addr)?;
                Ok(Signal::Dealloc)
            }
            Push(s) => {
                // For now, we just push the byte onto the stack without any checks. We can later add checks for stack overflow and other edge cases.
                self.mem.lock().unwrap()._push_checked(s.byte.into())?;
                Ok(Signal::Write)
            }
            Pop => {
                // For now, we just pop a byte from the stack without any checks. We can later add checks for stack underflow and other edge cases.
                let value = self.mem.lock().unwrap()._pop_checked()?;
                println!("Popped value: {}", value);
                Ok(Signal::Read)
            }
            Write((s, checked)) => {
                let addr = s.at;
                if *checked {
                    self.mem
                        .lock()
                        .unwrap()
                        ._write_at_addr_checked(addr, vec![s.byte.into()])?;
                } else {
                    self.mem
                        .lock()
                        .unwrap()
                        ._write_at_addr(addr, vec![s.byte.into()])?;
                }
                Ok(Signal::Write)
            }
            Read((s, checked)) => {
                let addr = s.at;
                // We read one byte or a complete struct depending on the request
                let value = if *checked {
                    self.mem.lock().unwrap()._read_at_checked(addr, 1)?
                } else {
                    self.mem.lock().unwrap()._read_at(addr, 1)?
                };
                println!("Read: {:?} -> {:?}", s, value);
                Ok(Signal::Read)
            }
            WriteAggr((s, checked)) => {
                let addr = s.first().expect("Expected at least one write request").at;
                let bytes: Vec<Byte> = s.iter().map(|req| req.byte).collect();
                if *checked {
                    self.mem.lock().unwrap()._write_at_addr_checked(
                        addr,
                        bytes.into_iter().map(Into::into).collect(),
                    )?;
                } else {
                    self.mem
                        .lock()
                        .unwrap()
                        ._write_at_addr(addr, bytes.into_iter().map(Into::into).collect())?;
                }
                Ok(Signal::Write)
            }
            ReadAggr((s, checked)) => {
                let addr = s.first().expect("Expected at least one read request").at;
                let len = s.len();
                let values = if *checked {
                    self.mem.lock().unwrap()._read_at_checked(addr, len)?
                } else {
                    self.mem.lock().unwrap()._read_at(addr, len)?
                };
                println!("ReadAggr: {:?} -> {:?}", s, values);
                Ok(Signal::Read)
            }
            Exit => {
                println!("Exit"); // We just print here, as this is handled externally
                Ok(Signal::Exit)
            }

            Empty => Ok(Signal::Empty),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_exec_debug() {
        // This test is just to check that the exec function can be called without panicking, and that the debug signal is returned correctly.
        let mut process = super::Process {
            pid: 1,
            mem: std::sync::Arc::new(std::sync::Mutex::new(crate::mem::Memory::new())),
            context: super::ProcessContext::default(),
        };

        let signal = process._exec(&super::Command::Debug).unwrap();
        assert_eq!(signal as u8, super::Signal::Debug as u8);
    }

    #[test]
    fn test_exec_alloc_and_dealloc() {
        let mut process = super::Process {
            pid: 1,
            mem: std::sync::Arc::new(std::sync::Mutex::new(crate::mem::Memory::new())),
            context: super::ProcessContext::default(),
        };

        let alloc_req = super::_AllocReq {
            byte: 42,
            at: Some(0x5000),
            label: None,
        };

        let signal = process._exec(&super::Command::Alloc(alloc_req)).unwrap();
        assert_eq!(signal as u8, super::Signal::Alloc as u8);

        let dealloc_req = super::_DeallocReq { at: 0x5000 };

        let signal = process
            ._exec(&super::Command::Dealloc(dealloc_req))
            .unwrap();
        assert_eq!(signal as u8, super::Signal::Dealloc as u8);
    }

    #[test]
    fn test_exec_write_and_read() {
        let mut process = super::Process {
            pid: 1,
            mem: std::sync::Arc::new(std::sync::Mutex::new(crate::mem::Memory::new())),
            context: super::ProcessContext::default(),
        };

        let _prealloc_req = super::_AllocReq {
            byte: 0,
            at: Some(0x5000),
            label: None,
        };

        process
            ._exec(&super::Command::Alloc(_prealloc_req))
            .unwrap();

        let write_req = super::_WriteReq {
            byte: 42,
            at: 0x5000,
        };

        let signal = process
            ._exec(&super::Command::Write((write_req, false)))
            .unwrap();
        assert_eq!(signal as u8, super::Signal::Write as u8);

        let read_req = super::_ReadReq { at: 0x5000 };

        let signal = process
            ._exec(&super::Command::Read((read_req, false)))
            .unwrap();
        assert_eq!(signal as u8, super::Signal::Read as u8);
    }

    #[test]
    fn test_exec_exit() {
        let mut process = super::Process {
            pid: 1,
            mem: std::sync::Arc::new(std::sync::Mutex::new(crate::mem::Memory::new())),
            context: super::ProcessContext::default(),
        };

        let signal = process._exec(&super::Command::Exit).unwrap();
        assert_eq!(signal as u8, super::Signal::Exit as u8);
    }

    #[test]
    fn test_exec_alloc_struct() {
        let mut process = super::Process {
            pid: 1,
            mem: std::sync::Arc::new(std::sync::Mutex::new(crate::mem::Memory::new())),
            context: super::ProcessContext::default(),
        };

        let alloc_struct_req = super::_AllocStructReq {
            fields: vec![("field1".to_string(), 42), ("field2".to_string(), 84)],
            at: Some(0x5000),
            label: Some("my_struct".to_string()),
        };

        let signal = process
            ._exec(&super::Command::AllocStruct(alloc_struct_req))
            .unwrap();
        assert_eq!(signal as u8, super::Signal::Alloc as u8);
    }

    #[test]
    fn test_exec_dealloc_struct() {
        let mut process = super::Process {
            pid: 1,
            mem: std::sync::Arc::new(std::sync::Mutex::new(crate::mem::Memory::new())),
            context: super::ProcessContext::default(),
        };

        let alloc_struct_req = super::_AllocStructReq {
            fields: vec![("field1".to_string(), 42), ("field2".to_string(), 84)],
            at: Some(0x5000),
            label: Some("my_struct".to_string()),
        };
        process
            ._exec(&super::Command::AllocStruct(alloc_struct_req))
            .unwrap();

        let dealloc_struct_req = super::_DeallocReq { at: 0x5000 };

        let signal = process
            ._exec(&super::Command::Dealloc(dealloc_struct_req))
            .unwrap();
        assert_eq!(signal as u8, super::Signal::Dealloc as u8);
    }

    #[test]
    fn test_exec_noop() {
        let mut process = super::Process {
            pid: 1,
            mem: std::sync::Arc::new(std::sync::Mutex::new(crate::mem::Memory::new())),
            context: super::ProcessContext::default(),
        };

        let signal = process._exec(&super::Command::Empty).unwrap();
        assert_eq!(signal as u8, super::Signal::Empty as u8);
    }
}
