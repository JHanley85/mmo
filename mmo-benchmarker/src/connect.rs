//! An example of hooking up stdin/stdout to either a TCP or UDP stream.
//!
//! This example will connect to a socket address specified in the argument list
//! and then forward all data read on stdin to the server, printing out all data
//! received on stdout. An optional `--udp` argument can be passed to specify
//! that the connection should be made over UDP instead of TCP, translating each
//! line entered on stdin to a UDP packet to be sent to the remote address.
//!
//! Note that this is not currently optimized for performance, especially
//! around buffer management. Rather it's intended to show an example of
//! working with a client.
//!
//! This example can be quite useful when interacting with the other examples in
//! this repository! Many of them recommend running this as a simple "hook up
//! stdin/stdout to a server" to get up and running.

extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate byteorder;

use std::env;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::thread;

use futures::sync::mpsc;
use futures::{Sink, Future, Stream};
use tokio_core::reactor::Core;
use std::str;


use std::io::Cursor;


fn main() {
     env_logger::init();
    // Determine if we're going to run in TCP or UDP mode
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let tcp = match args.iter().position(|a| a == "--udp") {
        Some(i) => {
            args.remove(i);
            false
        }
        None => true,
    };

    // Parse what address we're going to connect to
    let addr = args.first().unwrap_or_else(|| {
        panic!("this program requires at least one argument")
    });
    let addr = addr.parse::<SocketAddr>().unwrap();

    // Create the event loop and initiate the connection to the remote server
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Right now Tokio doesn't support a handle to stdin running on the event
    // loop, so we farm out that work to a separate thread. This thread will
    // read data (with blocking I/O) from stdin and then send it to the event
    // loop over a standard futures channel.
    let (stdin_tx, stdin_rx) = mpsc::channel(0);
    thread::spawn(|| read_stdin(stdin_tx));
    let stdin_rx = stdin_rx.map_err(|_| panic!()); // errors not possible on rx

    // Now that we've got our stdin read we either set up our TCP connection or
    // our UDP connection to get a stream of bytes we're going to emit to
    // stdout.
    let stdout = if tcp {
        tcp::connect(&addr, &handle, Box::new(stdin_rx))
    } else {
        udp::connect(&addr, &handle, Box::new(stdin_rx))
    };

    // And now with our stream of bytes to write to stdout, we execute that in
    // the event loop! Note that this is doing blocking I/O to emit data to
    // stdout, and in general it's a no-no to do that sort of work on the event
    // loop. In this case, though, we know it's ok as the event loop isn't
    // otherwise running anything useful.
    let mut out = io::stdout();
    core.run(stdout.for_each(|chunk| {
        out.write_all(&chunk)
    })).unwrap();
}

mod tcp {
    use std::io::{self, Read, Write};
    use std::net::{SocketAddr, Shutdown};

    use bytes::{BufMut, BytesMut};
    use futures::prelude::*;
    use tokio_core::net::TcpStream;
    use tokio_core::reactor::Handle;
    use tokio_io::{AsyncRead, AsyncWrite};
    use tokio_io::codec::{Encoder, Decoder};

    pub fn connect(addr: &SocketAddr,
                   handle: &Handle,
                   stdin: Box<Stream<Item = Vec<u8>, Error = io::Error>>)
        -> Box<Stream<Item = BytesMut, Error = io::Error>>
    {
        let tcp = TcpStream::connect(addr, handle);
        let handle = handle.clone();

        // After the TCP connection has been established, we set up our client
        // to start forwarding data.
        //
        // First we use the `Io::framed` method with a simple implementation of
        // a `Codec` (listed below) that just ships bytes around. We then split
        // that in two to work with the stream and sink separately.
        //
        // Half of the work we're going to do is to take all data we receive on
        // `stdin` and send that along the TCP stream (`sink`). The second half
        // is to take all the data we receive (`stream`) and then write that to
        // stdout. We'll be passing this handle back out from this method.
        //
        // You'll also note that we *spawn* the work to read stdin and write it
        // to the TCP stream. This is done to ensure that happens concurrently
        // with us reading data from the stream.
        Box::new(tcp.map(move |stream| {
            let stream = CloseWithShutdown(stream);
            let (sink, stream) = stream.framed(Bytes).split();
            let copy_stdin = stdin.forward(sink)
                .then(|result| {
                    if let Err(e) = result {
                        panic!("failed to write to socket: {}", e)
                    }
                    Ok(())
                });
            handle.spawn(copy_stdin);
            stream
        }).flatten_stream())
    }

    /// A small adapter to layer over our TCP stream which uses the `shutdown`
    /// syscall when the writer side is shut down. This'll allow us to correctly
    /// inform the remote end that we're done writing.
    struct CloseWithShutdown(TcpStream);

    impl Read for CloseWithShutdown {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.read(buf)
        }
    }

    impl AsyncRead for CloseWithShutdown {}

    impl Write for CloseWithShutdown {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.0.flush()
        }
    }

    impl AsyncWrite for CloseWithShutdown {
        fn shutdown(&mut self) -> Poll<(), io::Error> {
            self.0.shutdown(Shutdown::Write)?;
            Ok(().into())
        }
    }

    /// A simple `Codec` implementation that just ships bytes around.
    ///
    /// This type is used for "framing" a TCP stream of bytes but it's really
    /// just a convenient method for us to work with streams/sinks for now.
    /// This'll just take any data read and interpret it as a "frame" and
    /// conversely just shove data into the output location without looking at
    /// it.
    struct Bytes;

    impl Decoder for Bytes {
        type Item = BytesMut;
        type Error = io::Error;

        fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<BytesMut>> {
            if buf.len() > 0 {
                let len = buf.len();
                Ok(Some(buf.split_to(len)))
            } else {
                Ok(None)
            }
        }

        fn decode_eof(&mut self, buf: &mut BytesMut) -> io::Result<Option<BytesMut>> {
            self.decode(buf)
        }
    }

    impl Encoder for Bytes {
        type Item = Vec<u8>;
        type Error = io::Error;

        fn encode(&mut self, data: Vec<u8>, buf: &mut BytesMut) -> io::Result<()> {
            buf.put(&data[..]);
            Ok(())
        }
    }
}

mod udp {
    use std::io;
    use std::net::SocketAddr;

    use bytes::BytesMut;
    use futures::{Future, Stream};
    use tokio_core::net::{UdpCodec, UdpSocket};
    use tokio_core::reactor::Handle;
    use byteorder::{ByteOrder, LittleEndian,WriteBytesExt};
    

const MSG_PING: u8=0;
const MSG_WORLD: u8=1;
const MSG_AVATAR: u8=2;
const MSG_BYTE_ARRAY: u8=4;
//Server requests

const SR_TIME:u8=0;
const SR_REGISTER:u8=1;
const SR_ACK:u8=2;
const SR_CALLBACK_UPDATE:u8=3;
const SR_RPC:u8=4;
const SR_PROPERTYREP:u8=5;
const SR_FUNCREP:u8=6;
const SR_JOINED:u8=7;
const SR_CLOSED:u8=8;
//message Relevancy
const COND_INITIALONLY:u8=0; // - This property will only attempt to send on the initial bunch
const COND_OWNERONLY:u8=1; // - This property will only send to the actor's owner
const COND_SKIPOWNER:u8=2; // - This property send to every connection EXCEPT the owner
const COND_SIMULATEDONLY:u8=3; // - This property will only send to simulated actors
const COND_AUTONOMOUSONLY:u8=4; // - This property will only send to autonomous actors
const COND_SIMULATEDORPHYSICS:u8=5; //- This property will send to simulated OR bRepPhysics actors
const COND_INITIALOROWNER:u8=6; // - This property will send on the initial packet, or to the actors owner
const COND_CUSTOM:u8=7; // - This property has no particular condition, but wants the ability to toggle on/off via SetCustomIsActiveOverride
const COND_NONE:u8=8; // - This property will send to sender, and all listeners
const COND_SKIP:u8=0;

     pub fn register()->Vec<u8>{
         info!("REGISTER send");
        let mut wtr:Vec<u8>=vec![0;6];
        LittleEndian::write_u32(&mut wtr, 151u32);
        let mut out = vec![0;6];
        out[0] =MSG_WORLD;
        out[1] =SR_REGISTER;
        out.append(&mut wtr);
        return out;
     }
     fn sr_property()->Vec<u8>{
         info!("PROPERTY send");
        let mut wtr:Vec<u8>=vec![0;6];
        LittleEndian::write_u32(&mut wtr, 151u32);
        let mut out = vec![0;6];
        out[0] =MSG_WORLD;
        out[1] =SR_PROPERTYREP;
        return out;
     }
     fn sr_time()->Vec<u8>{
         info!("TIME send");
        let mut wtr:Vec<u8>=vec![0;6];
        LittleEndian::write_u32(&mut wtr, 151u32);
        let mut out = vec![0;6];
        out[0] =MSG_WORLD;
        out[1] =SR_TIME;
        return out;
     }
     fn sr_ack()->Vec<u8>{
         info!("ACK send");
        let mut wtr:Vec<u8>=vec![0;6];
        LittleEndian::write_u32(&mut wtr, 151u32);
        let mut out = vec![0;6];
        out[0] =MSG_WORLD;
        out[1] =SR_ACK;
        return out;
     }
     fn sr_callback()->Vec<u8>{
         info!("CALLBACK send");
        let mut wtr:Vec<u8>=vec![0;6];
        LittleEndian::write_u32(&mut wtr, 151u32);
        let mut out = vec![0;6];
        out[0] =MSG_WORLD;
        out[1] =SR_CALLBACK_UPDATE;
        return out;
     }
     fn sr_rpc()->Vec<u8>{
         info!("RPC send");
        let mut wtr:Vec<u8>=vec![0;6];
        LittleEndian::write_u32(&mut wtr, 151u32);
        let mut out = vec![0;6];
        out[0] =MSG_WORLD;
        out[1] =SR_RPC;
        return out;
     }
     fn sr_func()->Vec<u8>{
         info!("FUNC send");
        let mut wtr:Vec<u8>=vec![0;6];
        LittleEndian::write_u32(&mut wtr, 151u32);
        let mut out = vec![0;6];
        out[0] =MSG_WORLD;
        out[1] =SR_FUNCREP;
        return out;
     }
     fn sr_close()->Vec<u8>{
         info!("CLOSE send");
        let mut out = vec![0;2];
        out[0] =MSG_WORLD;
        out[1] =SR_CLOSED;
        return out;
     }
    pub fn connect(&addr: &SocketAddr,
                   handle: &Handle,
                   stdin: Box<Stream<Item = Vec<u8>, Error = io::Error>>)
        -> Box<Stream<Item = BytesMut, Error = io::Error>>
    {
        // We'll bind our UDP socket to a local IP/port, but for now we
        // basically let the OS pick both of those.
        let addr_to_bind = if addr.ip().is_ipv4() {
            "0.0.0.0:0".parse().unwrap()
        } else {
            "[::]:0".parse().unwrap()
        };
        let udp = UdpSocket::bind(&addr_to_bind, handle)
            .expect("failed to bind socket");

        // Like above with TCP we use an instance of `UdpCodec` to transform
        // this UDP socket into a framed sink/stream which operates over
        // discrete values. In this case we're working with *pairs* of socket
        // addresses and byte buffers.
        let (sink, stream) = udp.framed(Bytes).split();
        let methods = vec![
            String::from("register"),
            String::from("property"),
            String::from("ack"),
            String::from("func"),
            String::from("callback"),
            String::from("time"),
            String::from("rpc"),
            String::from("close"),
        ];

        info!( "methods are {:?}",&methods[0..]);
        // All bytes from `stdin` will go to the `addr` specified in our
        // argument list. Like with TCP this is spawned concurrently
        handle.spawn(stdin.map(move |chunk| {
            let mut out:Vec<u8>= chunk[0..].to_vec();
            let msg = String::from_utf8(out.clone().to_vec()).unwrap();
            if msg.trim()==String::from("register"){
                out=register();
            }else if msg.trim()==String::from("property"){
                out=sr_property();
            }else if msg.trim()==String::from("ack"){
                out=sr_ack();
            }else if msg.trim()==String::from("func"){
                out=sr_func();
            }else if msg.trim()==String::from("callback"){
                out=sr_callback();
            }else if msg.trim()==String::from("time"){
                out=sr_time();
            }else if msg.trim()==String::from("rpc"){
                out=sr_rpc();
            }else if msg.trim()==String::from("close"){
                out=sr_close();
            }
            info!("Sending out {} {}",out[0],out[1]);
            return (addr, out.to_vec())
        }).forward(sink).then(|result| {
            if let Err(e) = result {
                panic!("failed to write to socket: {}", e)
            }
            Ok(())
        }));

        // With UDP we could receive data from any source, so filter out
        // anything coming from a different address
        Box::new(stream.filter_map(move |(src, chunk)| {
          
            if src == addr || true {
                   let d:&[u8] = &chunk[0..];
                   let s = String::from_utf8_lossy(d);
                   println!("result: {}", s);
                Some(d.into())
            } else {
                None
            }
        }))
    }

    struct Bytes;

    impl UdpCodec for Bytes {
        type In = (SocketAddr, Vec<u8>);
        type Out = (SocketAddr, Vec<u8>);

        fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
            Ok((*addr, buf.to_vec()))
        }

        fn encode(&mut self, (addr, buf): Self::Out, into: &mut Vec<u8>) -> SocketAddr {
            into.extend(buf);
            addr
        }
    }
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
fn read_stdin(mut tx: mpsc::Sender<Vec<u8>>) {
    let mut stdin = io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf) {
            Err(_) |
            Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx = match tx.send(buf).wait() {
            Ok(tx) => tx,
            Err(_) => break,
        };
    }
}