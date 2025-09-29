use sosim::{
    // allocator, fault,
    // mem::addr::Addr,
    paging::{PTEntry},
    lang::script::parse_src
};
use std::{fs::{read_to_string}, mem::size_of};

#[cfg(any(
    all(feature = "bit16", feature = "bit32"),
    all(feature = "bit16", feature = "bit64"),
    all(feature = "bit16", feature = "bit8"),
    all(feature = "bit32", feature = "bit64"),
    all(feature = "bit32", feature = "bit8"),
    all(feature = "bit64", feature = "bit8"),
))]
compile_error!("Only one of bit8, bit16, bit32, or bit64 features can be enabled at a time.");

// #[derive(Deserialize, Debug)]
// struct Dummy {
//     family: u16,
// }

fn main() {
    println!("{}", size_of::<PTEntry>());
    println!("align of S: {}", std::mem::align_of::<PTEntry>());
    let contents = read_to_string("lang_test/first.sos").expect("File reading error...");
    // let stdout = stdout();
    println!("{:?}", parse_src(contents).unwrap());
}
