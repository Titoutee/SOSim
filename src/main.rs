use clap::{Parser, Subcommand};
use sosim::{
    lang::{script::parse_src, toplevel::TopLevel},
    // allocator, fault,
    // mem::addr::Addr,
    mem::paging::PTEntry,
};
use std::{
    fs::read_to_string,
    net::{Ipv4Addr, SocketAddrV4},
};

#[cfg(any(
    all(feature = "bit16", feature = "bit32"),
    all(feature = "bit16", feature = "bit64"),
    all(feature = "bit16", feature = "bit8"),
    all(feature = "bit32", feature = "bit64"),
    all(feature = "bit32", feature = "bit8"),
    all(feature = "bit64", feature = "bit8"),
))]
compile_error!("Only one of bit8, bit16, bit32, or bit64 features can be enabled at a time.");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    file: Option<String>, // If Some(path), then simulator launched in static interpreter mode
    // If None, then simulator launched in dynamic toplevel interpreter mode
    #[arg(short, long)]
    socket: Option<SocketAddrV4>, // Parsed from standard string parsing format: a.d.d.r:port
}

#[tokio::main]
async fn main() {
    // println!("{}", size_of::<PTEntry>());
    // println!("align of S: {}", std::mem::align_of::<PTEntry>());

    let cli = Cli::parse();
    println!("Parsing arguments [...]");

    if let Some(path) = cli.file {
        let contents = read_to_string(path).expect("File reading error...");
        println!("{:?}", parse_src(contents).unwrap());
    } else {
        TopLevel::_spawn(cli.socket).await;
    }
}
