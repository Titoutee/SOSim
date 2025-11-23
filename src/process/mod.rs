//! Processes

use crate::mem::paging::PageTable;

pub struct ProcContext {}

pub struct Process {
    ctxt: ProcContext,
    pt: PageTable,
}
