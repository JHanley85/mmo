extern crate futures;
extern crate futures_io;
extern crate futures_mio;

extern crate clap;
use std::net::SocketAddr;
use std::net::UdpSocket;
use clap::{Arg, App};
use futures::Future;
use futures_io::{copy, TaskIo};
use futures::stream::Stream;
fn app() -> App<'static, 'static> {
    App::new("mmo-mumble")
        .version("0.1.0")
        .about("Voice client bot!")
        .author("Alex Rozgo")
        .arg(Arg::with_name("addr").short("a").long("address").help("Host to connect to address:port").takes_value(true))

}
fn main() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
        socket.connect("127.0.0.1:8080").expect("connect function failed");
        //let mut socket = UdpSocket::bind("127.0.0.1:8080")?;
socket.send(&[250,250,250]).expect("couldn't send message");
        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 10];
        let (amt, src) = socket.recv_from(&mut buf)?;

        // Redeclare `buf` as slice of the received data and send reverse data back to origin.
        let buf = &mut buf[..amt];
        buf.reverse();
        socket.send_to(buf, &src)?;
    } // the socket is closed here
    Ok(())
}
fn main3() {
  
    let matches = app().get_matches();

    let addr_str = matches.value_of("addr").unwrap_or("127.0.0.1:8080");
    let addr = addr_str.parse::<SocketAddr>().unwrap();

    let mut l = futures_mio::Loop::new().unwrap();

    let server = l.handle().tcp_listen(&addr);

    let done = server.and_then(move |socket| {
        println!("Listening on: {}", addr);
        socket.incoming().for_each(|(socket, addr)| {
            
            println!("Incoming on: {}", addr);
            let io = TaskIo::new(socket);
            let pair = io.map(|io| io.split());
            let amt = pair.and_then(|(reader, writer)| {
                copy(reader, writer)
            });
            amt.map(move |amt| {
                println!("wrote {} bytes to {}", amt, addr)
            }).forget();

            Ok(())
        })
    });
    l.run(done).unwrap();
}


