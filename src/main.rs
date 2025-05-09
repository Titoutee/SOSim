use sosim::{fault, allocator, mem::{addr::Addr, paging::{FullPTEntry}}};
use std::{mem::size_of};

#[cfg(any(
    all(feature = "bit16", feature = "bit32"),
    all(feature = "bit16", feature = "bit64"),
    all(feature = "bit16", feature = "bit8"),
    all(feature = "bit32", feature = "bit64"),
    all(feature = "bit32", feature = "bit8"),
    all(feature = "bit64", feature = "bit8"),
))]
compile_error!("Only one of bit8, bit16, bit32, or bit64 features can be enabled at a time.");

fn main() {
    println!("{}", size_of::<FullPTEntry>());    
    println!("align of S: {}", std::mem::align_of::<FullPTEntry>());
}
