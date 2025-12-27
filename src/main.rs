use clap::Parser;
use sosim::{
    Machine,
    lang::{script::parse_src, toplevel::TopLevel},
};
use std::{fs::read_to_string, net::SocketAddrV4};

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

#[allow(unused)]
#[tokio::main]
async fn main() {
    // println!("{}", size_of::<PTEntry>());
    // println!("align of S: {}", std::mem::align_of::<PTEntry>());
    let machine = Machine::new();

    // Process adding...

    println!("Max. concurrent processes number: {}", num_cpus::get());

    let cli = Cli::parse();
    println!("Parsing arguments [...]");

    if let Some(path) = cli.file {
        println!("[File parsing mode]");
        let contents = read_to_string(path).expect("File reading error...");
        println!("{:?}", parse_src(contents).unwrap());
    } else {
        println!("[Toplevel interpreting mode]");
        println!("[Server startup]");

        TopLevel::_spawn(cli.socket).await.unwrap();
    }
}
