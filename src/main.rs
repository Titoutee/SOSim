use sosim::{fault, allocator, mem::{addr::Addr, paging::{FullPTEntry}}};
use std::{mem::size_of};

fn main() {
    println!("{}", size_of::<FullPTEntry>());    
    println!("align of S: {}", std::mem::align_of::<FullPTEntry>());
}
