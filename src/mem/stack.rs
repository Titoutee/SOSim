use super::Ram;
use super::VirtualAddress;
use crate::paging::PageDirectory;

pub struct Stack<'a> {
    pub top: VirtualAddress,       // current top of the stack (grows downward)
    pub base: VirtualAddress,      // fixed bottom of the stack
    pub size: usize,               // total stack size in bytes
    pub page_table: &'a PageDirectory, // memory mapping
    pub ram: &'a Ram,                  // physical memory backing
}
