#![allow(unused)]
use std::io::prelude::*;
use std::fs::File;

#[macro_use]
extern crate clap;
use clap::{Arg, App};

extern crate anevicon_core;
use anevicon_core::summary::TestSummary;
use anevicon_core::testing::send;
fn main(){
     env_logger::init();
    let matches = app().get_matches();
    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:8080");
    let addr = addr.parse::<SocketAddr>().unwrap();

    
    // Setup the socket connected to the example.com domain
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.connect(addr).unwrap();

    let packet = vec![0; 32768];
    let mut summary = TestSummary::default();
    // Execute a test that will send one thousand packets
    // each containing 32768 bytes.
    for _ in 0..1000 {
        if let Err(error) = send(&socket, &packet, &mut summary) {
            panic!("{}", error);
        }
    }


    println!(
        "The total seconds passed: {}",
        summary.time_passed().as_secs()
    );

}





fn app()-> App<'static, 'static> {
    //! get args and info
    App::new("mmo-server")
        .version("0.1.0")
        .about("Simulates traffic")
        .author("Jack Hanley")
        .arg(Arg::with_name("addr")
            .short("a")
            .long("address")
            .help("Host to connect to address:port")
            .takes_value(true))
}

