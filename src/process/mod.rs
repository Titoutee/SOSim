//! Processes

use super::{paging::PageTable};

pub struct ProcContext {
    
}

pub struct Process {
    ctxt: ProcContext,
    pt: Box<PageTable>,
}