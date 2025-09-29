//! Processes

use super::{paging::PageTable};

pub struct ProcContext {
    //dummy
}

pub struct Process {
    ctxt: ProcContext,
    pt: Box<PageTable>,
}