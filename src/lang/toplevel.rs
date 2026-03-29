//! Toplevel behaviour, which is just for conveniency as testing with files is a headbang.
//! Using a separate CLI for feeding in commands line by line is more convenient :D

use super::Command;
use crate::fault::Fault;
use crate::lang::parse_src;
use crate::mem::MemResult;
use crate::process::Process;
use bytes::BytesMut;
use std::io::{self, Write};
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const LOCALHOST: [u8; 4] = [127, 0, 0, 1];
const LOCALPORT: u16 = 6379; // Or any free

const NET_SOCK_CFG_PATH: &'static str = "toplevel/net_cfg";

pub type NetResult<T> = Result<T, std::io::Error>;

async fn _signal(top: &mut TopLevel, sig: u8) -> io::Result<usize> {
    top.stream.write(&sig.to_be_bytes()).await
}

// Main routine of the toplevel thread, which gets MiniLang commands from the external CLI and sends them to the parser in the order they arrive.
async fn _main(proc: &mut Process, mut top: TopLevel) -> MemResult<()> {
    loop {
        top.flush_stream().await;
        // println!("Try!!!");
        let inc = top._read().await;
        // println!("Command got");
        match inc {
            Some(i) => {
                println!("Got valid command!");
                match i {
                    Command::Exit => {
                        let _n = _signal(&mut top, 4)
                            .await
                            .map_err(|_| Fault::_from(crate::fault::FaultType::SignallingFault))?;
                        println!("[{} bytes written to the client]", _n);
                        println!("Client exited gracefully...");
                        break;
                    }
                    _ => {
                        let sig = proc._exec(&i)?;
                        let _n = _signal(&mut top, sig as u8)
                            .await
                            .map_err(|_| Fault::_from(crate::fault::FaultType::SignallingFault))?; // Serialization happens here
                        println!("[{} bytes written to the client]", _n as u8);
                    }
                }
            }
            None => {
                println!("Unknown problem occured");
                top.stream
                    .write(&(1 as u8).to_be_bytes())
                    .await
                    .ok()
                    .unwrap();
                continue;
            }
        };
    }
    Ok(())
}

/// A `TopLevel` really is just a separate thread which reads in standard input rather than a specified file.
///
/// An instance acts *in fine* as an interface between the client and the "server" (we mean the main thread) (even if we can't really talk about
/// a client-server architecture here).
/// The `TopLevel` is owned by the main thread.
pub struct TopLevel {
    pub stream: TcpStream,
    pub buffer: BytesMut,
}

impl<'a> TopLevel {
    async fn new(stream: TcpStream) -> NetResult<TopLevel> {
        Ok(TopLevel {
            stream,
            buffer: BytesMut::with_capacity(10), // Empty buffer at the point of creation
        })
    }

    async fn flush_stream(&mut self) {
        self.buffer.clear();
        // self.stream.readable().await;
    }

    // Reads in a single command (default = first in command list) which is more handy for toplevel interpreter mode
    pub async fn _read(&mut self) -> Option<Command> {
        // let string = String::new();
        let bytes_read = self.stream.read_buf(&mut self.buffer).await.ok()?;

        if bytes_read == 0 {
            println!("Read 0 bytes!!");
            return None;
        }

        // Ok-ish clone
        let cmds = parse_src(
            String::from_utf8(self.buffer.to_vec())
                .ok()
                .expect("utf-8 error"),
        )
        .ok()?;
        println!("{:?}", cmds);
        // self.flush_stream().await;
        cmds.get(0).map(|x| (*x).clone())
    }

    // Spawns a new command server which the external toplevel binds to
    pub async fn _spawn(bind: Option<SocketAddrV4>, proc: &mut Process) -> NetResult<()> {
        let socket = if let Some(b) = bind {
            b
        } else {
            let addr = Ipv4Addr::from_octets(LOCALHOST);
            SocketAddrV4::new(addr, LOCALPORT)
        };

        // Serialize socket config

        // Manual socket serializing:
        //      - first line: address
        //      - second line: port

        let mut f = std::fs::File::create(NET_SOCK_CFG_PATH)
            .expect("Error opening/creating socket config file...");

        let ser = format!("{}\n{}", socket.ip(), socket.port());
        let n = f.write(ser.as_bytes())?;

        println!(
            "[Wrote {} bytes to file at path \"{}\"]",
            n, NET_SOCK_CFG_PATH
        );

        // Net

        let listener = TcpListener::bind(socket).await?;
        let s = listener.accept().await.expect("Connection failed..."); // Blocking
        println!("Client of socket address {}", s.1);
        let top = TopLevel::new(s.0).await?;

        _main(proc, top).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::lang::parse_src;

    #[test]
    fn test_parse() {
        let src = "alloc 1 at 0; alloc 2 at 1; alloc 3 at 2; dbg; dealloc at 1; dbg;";
        let cmds = parse_src(src.to_string()).unwrap();
        println!("{:?}", cmds);
    }

    #[test]
    fn test_parse_expr() {
        let src = "alloc 1+2*3 at 0; alloc (1+2)*3 at 1; dbg;";
        let cmds = parse_src(src.to_string()).unwrap();
        println!("{:?}", cmds);
    }

    #[test]
    fn test_parse_struct() {
        let src = "struct a {a: 1, b: 2} at 0; dbg;";
        let cmds = parse_src(src.to_string()).unwrap();
        println!("{:?}", cmds);
    }

    #[test]
    #[should_panic]
    fn test_parse_unknown() {
        let src = "unknown command";
        let cmds = parse_src(src.to_string()).unwrap();
        println!("{:?}", cmds);
    }
}
