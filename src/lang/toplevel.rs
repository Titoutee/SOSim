//! Toplevel behaviour, which is just for conveniency as testing with files is a headbang.
//! Using a separate CLI for feeding in commands line by line is more convenient lol :D

use super::Command;
use crate::lang::parse_src;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::thread;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

const LOCALHOST: [u8; 4] = [127, 0, 0, 1];
const LOCALPORT: u16 = 6378; // Or any free

pub type NetResult<T> = Result<T, std::io::Error>;

// Main routine of the toplevel thread, which gets MiniLang commands from the external CLI and sends them to the parser in order.
async fn _main(top: &mut TopLevel, s: &mut TcpStream) -> Option<()> {
    None
}

// Changes toplevel internal buffer
async fn _read(top: &mut TopLevel, s: &mut TcpStream) -> Option<Vec<Command>> {
    let bytes_read = s.read_buf(&mut top.buffer).await.ok().map(|a| a)?;

    if bytes_read == 0 {
        return None;
    }

    // Ok-ish clone
    let cmds = parse_src(String::from_utf8(top.buffer.clone()).ok()?).ok()?;
    Some(cmds)
}

/// A `TopLevel` really is just a separate thread which reads in standard input rather than a specified file.
///
/// An instance acts *in fine* as an interface between the client and the "server" (we mean the main thread) (even if we can't really talk about
/// a client-server architecture here).
/// The `TopLevel` is owned by the main thread.
pub struct TopLevel {
    listener: TcpListener,
    buffer: Vec<u8>,
}

impl TopLevel {
    async fn new(bind: SocketAddrV4) -> NetResult<TopLevel> {
        let listener = TcpListener::bind(bind).await?;
        Ok(TopLevel {
            listener,
            buffer: vec![], // Empty buffer at the point of creation
        })
    }

    pub async fn _spawn(bind: Option<SocketAddrV4>) -> NetResult<()> {
        let top = TopLevel::new(if let Some(b) = bind {
            b
        } else {
            let addr = Ipv4Addr::from_octets(LOCALHOST);
            SocketAddrV4::new(addr, LOCALPORT)
        })
        .await
        .expect("Error while spawning toplevel");

        // Accepts a single connection from foreign toplevel client
        let s = top.listener.accept().await.expect("Connection failed..."); // Blocking

        println!(
            "Received connection from separate toplevel:\n{:#?}; {}",
            s.0, s.1
        );

        thread::spawn(move || todo!());
        Ok(())
    }
}
