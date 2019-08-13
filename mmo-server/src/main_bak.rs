
//! An UDP echo server that just sends back everything that it receives.
//!
//! If you're on unix you can test this out by in one terminal executing:
//!
//!     cargo run --example echo-udp
//!
//! and in another terminal you can run:
//!
//!     cargo run --example connect -- --udp 127.0.0.1:8080
//!
//! Each line you type in to the `nc` terminal should be echo'd back to you!
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#[macro_use]
extern crate log;
extern crate env_logger;

use std::fmt::{Formatter,Error,Display};
#[macro_use]
extern crate clap;
use clap::{Arg, App};

#[macro_use]
extern crate futures;
extern crate tokio;

use tokio::io;
use tokio::net::TcpStream;
use tokio::net::UdpSocket;
use tokio::prelude::*;
extern crate byteorder;
extern crate time;
extern crate rand;
extern crate mentat;

use std::{io as stdio};
use std::net::SocketAddr;
use std::string::String;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::{Future, Poll};
// use tokio_core::reactor::Core;

use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use byteorder::{ByteOrder, LittleEndian,WriteBytesExt};

mod server;
use server::Server;
use server::Listener;
fn app()-> App<'static, 'static> {
    //! get args and info
    App::new("mmo-server")
        .version("1.1.0")
        .about("Rust UE4 mmo server for help dial 1-800-273-8255")
        .author("Jack Hanley")
        .arg(Arg::with_name("addr")
            .short("a")
            .long("address")
            .help("Host to connect to address:port")
            .takes_value(true))
}

fn main() {
    //! Main.
    env_logger::init();
    let matches = app().get_matches();
    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:8080");
    let addr = addr.parse::<SocketAddr>().unwrap();
    // Create the event loop that will drive this server, and also bind the
    // socket we'll be listening to.
    // let mut l = Core::new().unwrap();
    // let handle = l.handle();
   // let udpsocket =  UdpSocket::bind(&addr).unwrap();
   // l.run(Server::new(udpsocket));
   let server = Server::new(addr);
   let streamer  = Listener::new(server);
   tokio::run(streamer);
}