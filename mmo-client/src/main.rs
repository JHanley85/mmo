//! A "souped up" echo server example.
//!
//! Very similar to the example at https://tokio.rs

#![feature(proc_macro, proc_macro_non_items, generators)]
//#![feature(use_extern_macros)]
#![feature(stmt_expr_attributes)]
//#![feature(proc_macro_expr)]
extern crate futures_await as futures;
extern crate clap;
extern crate tokio_core;
extern crate tokio_io;
use clap::{Arg, App};
use std::io::{self, BufReader};

use futures::prelude::*;
use tokio_core::net::{TcpListener, TcpStream};
use tokio_core::reactor::Core;
use tokio_io::{AsyncRead};

fn main() {
	let matches = App::new("mmo-server")
        .version("0.1.0")
        .about("Simulates a slice of universe!")
        .author("Alex Rozgo")
        .arg(Arg::with_name("addr")
            .short("a")
            .long("address")
            .help("Host to connect to address:port")
            .takes_value(true))
        .get_matches();
    // Create the event loop that will drive this server
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Bind the server's socket
    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:8080");
	let addr = addr.parse::<std::net::SocketAddr>().unwrap();
    //let addr = "127.0.0.1:12345".parse().unwrap();
    let tcp = TcpListener::bind(&addr, &handle).expect("failed to bind listener");
    println!("listening for connections on {}",
             tcp.local_addr().unwrap());

    let server = async_block! {
        #[async]
        for (client, _) in tcp.incoming() {
            handle.spawn(handle_client(client).then(|result| {
                match result {
                    Ok(n) => println!("wrote {} bytes", n),
                    Err(e) => println!("IO error {:?}", e),
                }
                Ok(())
            }));
        }

        Ok::<(), io::Error>(())
    };
    core.run(server).unwrap();
}

#[async]
fn handle_client(socket: TcpStream) -> io::Result<u64> {
    let (reader, mut writer) = socket.split();
    let input = BufReader::new(reader);

    let mut total = 0;

	#[async]
    for line in tokio_io::io::lines(input) {
        println!("got client line: {}", line);
        total += line.len() as u64;
        writer = futures::prelude::await!(tokio_io::io::write_all(writer, line))?.0;
    }

    Ok(total)
}