use sosim::{fault, allocator, mem::{addr::Addr, paging::PTEntry}};
use std::{mem::size_of};

fn main() {
    println!("{}", size_of::<PTEntry>());    
    println!("align of S: {}", std::mem::align_of::<PTEntry>());
}
