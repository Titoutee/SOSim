//! Toplevel behaviour, which is just for conveniency as testing with files is a headbang.
//! Using a separate CLI for feeding in commands line by line is more convenient lol :D

use std::net::{SocketAddrV4};
use std::thread;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use bytes::BytesMut;

async fn _main(top: &mut TopLevel, s: &mut TcpStream) -> Option<()> {
    let bytes_read = s.read_buf(&mut top.buffer).await.ok().map(|_|())?;
    None
}

/// A `TopLevel is really just a main routine which reads in standard input rather than a specified file.
///
/// An instance acts *in fine* as an interface between the client and the "server" (we mean the main thread) (even if we can't really talk about
/// a client-server architecture here).
/// The `TopLevel` is owned by the main thread.
pub struct TopLevel {
    listener: TcpListener,
    buffer: BytesMut,
}

impl TopLevel {
    pub async fn new(bind: SocketAddrV4) -> Result<TcpListener, std::io::Error> {
        TcpListener::bind(bind).await
    }

    /// Accepts one only client
    pub async fn _spawn(&self) {
        // let lock = false;
        // loop { // The loop permit to leave the main thread awake between two separate clients
        //
        // }
        let s = self.listener.accept().await;
        thread::spawn(move || todo!());
    }
}
