#[macro_use]
extern crate clap;
extern crate futures;
extern crate tokio_core;
extern crate byteorder;

use clap::{Arg, App};

use std::io;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::time::{Instant, Duration};

use futures::{Future, stream, Stream, Sink};
use tokio_core::net::{UdpSocket, UdpCodec};
use tokio_core::reactor::Core;

use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};

pub struct LineCodec;

impl UdpCodec for LineCodec {
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

pub struct Client {
    pub instant: Instant,
}
fn app()-> App<'static, 'static> {
    App::new("mmo-server")
        .version("0.1.0")
        .about("Simulates a slice of universe!")
        .author("Alex Rozgo")
        .arg(Arg::with_name("addr")
            .short("a")
            .long("address")
            .help("Host to connect to address:port")
            .takes_value(true))
        .arg(Arg::with_name("exp")
            .short("e")
            .long("expiration")
            .help("Connection expiration limit")
            .takes_value(true))
}


fn main() {
    let matches = app().get_matches();
    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:8080");
    let addr = addr.parse::<SocketAddr>().unwrap();
    let exp = value_t!(matches, "exp", u64).unwrap_or(5);

    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let udp_local_addr: SocketAddr = addr;
    let server_socket = UdpSocket::bind(&udp_local_addr, &handle).unwrap();
   
    let (udp_socket_tx, udp_socket_rx) = server_socket.framed(LineCodec).split();
    let expiration = Duration::from_secs(exp);
    let clients = &mut HashMap::<SocketAddr, Client>::new();
    println!("Waiting for clients on: {}", addr);
    let listen_task = udp_socket_rx.fold((clients, udp_socket_tx), |(clients, udp_socket_tx), (client_socket, msg)| {
            if msg.len()==0 {
               Error::new(ErrorKind::Other, "msg len");
            }
            
            println!("{}: {:?} bytes", client_socket, msg.len());
            if msg[0] == 0 { // this needs to be checked first. Port scanning panics.
                let mut rdr = std::io::Cursor::new(&msg[1..]);
                let uuid = rdr.read_u32::<LittleEndian>().unwrap();
                 println!("Client: {} UUID: {}", client_socket, uuid);
                 join(client_socket,uuid);
            }
            if !clients.contains_key(&client_socket) {
                println!("Connected: {} Online: {}", client_socket, clients.len() + 1);
            }
            clients.insert(client_socket, Client { instant: Instant::now() });

            let mut expired = Vec::new();
            for (client_socket, client) in clients.iter() {
                if client.instant.elapsed() > expiration {
                    expired.push(*client_socket);
                }
            }
            for client_socket in expired {
                clients.remove(&client_socket);
                println!("Expired: {} Online: {}", client_socket, clients.len());
            }

            let client_sockets: Vec<_> = clients.keys()
              //  .filter(|&&x| x != client_socket)
                .map(|k| *k).collect();
            stream::iter_ok::<_, ()>(client_sockets)
            .fold(udp_socket_tx, move |udp_socket_tx, client_socket| {
            // println!("{}: {:?}", client_socket, msg.clone());
                udp_socket_tx.send((client_socket, msg.clone()))
                .map_err(|_| ())
            })
            .map(|udp_socket_tx| (clients, udp_socket_tx))
            .map_err(|_| Error::new(ErrorKind::Other, "broadcasting to clients"))
    });

    if let Err(err) = core.run(listen_task) {
        println!("{}", err);
    }
}


fn join(client_socket: SocketAddr,uuid: u32){
    println!("Client: {} UUID: {}", client_socket, uuid);
    println!("This is where we'd send initial state stuff.");
}