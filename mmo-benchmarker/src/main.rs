
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
#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate clap;
use clap::{Arg, App};
extern crate futures;
#[macro_use]
extern crate tokio_core;
extern crate byteorder;
extern crate time;
extern crate rand;

use std::{env, io};
use std::net::SocketAddr;

use futures::{Future, Poll};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use std::time::{Instant, Duration};
use std::collections::HashMap;

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
const SR_AUTHORITY:u8=7;
const SR_STATE:u8=8;
const SR_JOINED:u8=9;
const SR_CLOSED:u8=10;
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



pub enum ServerError {
    Variant1,
    Variant2,
}
struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,
    connections:HashMap<u32,Client>,
}
pub type ServerRouteResult = Result<Vec<u8>,ServerError>;
trait Router{
     fn get_sender_id(&mut self,sender:SocketAddr)->u32;
    fn parse_route(&mut self,msg:Vec<u8>,addr:SocketAddr)->bool;
    fn register_connection(&mut self,msg:&[u8],peer:SocketAddr)->bool;
    fn rpc_message(&mut self,msg:&[u8],sender:SocketAddr)->bool;
    fn property_message(&mut self,msg:&[u8],sender:SocketAddr)->bool;
    fn callback_message(&mut self,msg:&[u8],sender:SocketAddr)->bool;
    fn function_message(&mut self,msg:&[u8],sender:SocketAddr)->bool;
    fn ping_message(&mut self,msg:&[u8],sender:SocketAddr)->bool;
    fn time_message(&mut self,msg:&[u8],sender:SocketAddr)->bool;
    fn ack_message(&mut self,msg:&[u8],sender:SocketAddr)->bool;
    fn closed_connection(&mut self,msg:&[u8],peer:SocketAddr)->bool;
    fn broadcast(&mut self,msg:&[u8],sender:SocketAddr,rep_condition:u8)->bool;
}
impl Router for Server{
    fn get_sender_id(&mut self,sender:SocketAddr)->u32{
         let data_source : &Vec<u32>  =  &self.connections.iter()
        .filter(|&(k,v)| v.addr == sender).map(|(k,v)| *k).collect();
        if(data_source.len() < 1){
            return 0;
        }else{
            return data_source[0];
        }
    }
    fn parse_route(&mut self,msg:Vec<u8>,peer:SocketAddr)->bool{
        let mut outmessage:Vec<u8> = msg.clone();
        let mut out_cond:u8=COND_NONE;
        if(msg.len() < 2){
            warn!("Rec'd small msg from {} {:?}",peer,&msg[0..]);
            return true;
        }
        let channel = msg[0];
        let system = msg[1];
        let senderid = self.get_sender_id(peer);
        debug!("C{} S{}, from {}",channel,system,senderid);
        let mut isSR = channel==MSG_WORLD;
        if channel==MSG_WORLD {
            match system {
                SR_REGISTER=>{
                    self.register_connection(&msg[2..],peer);
                    ()
                }
                SR_TIME=>{
                    if(senderid!=0){
                        self.time_message(&msg[2..],peer);
                    }
                    ()
                }
                SR_CALLBACK_UPDATE=>{
                    if(senderid!=0){
                        self.callback_message(&msg[2..],peer);
                    }()
                }
                SR_PROPERTYREP=>{
                    if(senderid!=0){
                        self.property_message(&msg[2..],peer);
                    }
                    ()
                }
                SR_FUNCREP=>{
                    if(senderid!=0){
                        self.function_message(&msg[2..],peer);
                    }
                    ()
                }
                SR_ACK=>{
                    if(senderid!=0){
                        self.ack_message(&msg[2..],peer);
                    }
                    ()
                }
                SR_RPC=>{
                    if(senderid!=0){
                        self.rpc_message(&msg[2..],peer);
                    }
                    ()
                }
                SR_CLOSED=>{
                    if(senderid!=0){
                        self.closed_connection(&msg[2..],peer);
                    }
                    ()
                }
                _=>{
                    isSR=false;
                    ()
                }
            }
        }
        return isSR;
    }
    fn closed_connection(&mut self,msg:&[u8],sender:SocketAddr)->bool{
        debug!("CLOSE Message Event from {}- {} bytes",sender,msg.len());
       let data_source : &Vec<u32>  =  &self.connections.iter()
        .filter(|&(k,v)| v.addr == sender).map(|(k,v)| *k).collect();
        for uid in data_source.iter(){
            &self.connections.remove(uid);
            let mut msg = vec![MSG_WORLD,SR_CLOSED];
            let mut wtr:Vec<u8>=vec![0;6];
            LittleEndian::write_u32(&mut wtr, *uid);
            msg.extend_from_slice(&wtr);
            self.broadcast(&msg[0..],sender,COND_SKIPOWNER);
        }
       
        return true;
    }
    fn register_connection(&mut self,msg:&[u8],sender:SocketAddr)->bool{
        debug!("REGISTER Message Event from {}- {} bytes",sender,msg.len());
        self.add_connection(8u32,sender);
        return true;
    }
    fn rpc_message(&mut self,msg:&[u8],sender:SocketAddr)->bool{
        debug!("RPC Message Event from {}- {} bytes",sender,msg.len());
        self.broadcast(&msg[0..],sender,COND_NONE);
        return true;

    }
    fn property_message(&mut self,msg:&[u8],sender:SocketAddr)->bool{
        debug!("PROPERTY Message Event from {}- {} bytes",sender,msg.len());
        self.broadcast(&msg[0..],sender,COND_NONE);
        return true;

    }
    fn callback_message(&mut self,msg:&[u8],sender:SocketAddr)->bool{
        debug!("CALLBACK Message Event from {}- {} bytes",sender,msg.len());
        self.broadcast(&msg[0..],sender,COND_NONE);
        return true;

    }
    fn function_message(&mut self,msg:&[u8],sender:SocketAddr)->bool{
        debug!("FUNCTION Message Event from {}- {} bytes",sender,msg.len());
        self.broadcast(&msg[0..],sender,COND_NONE);
        return true;
        }
    fn ping_message(&mut self,msg:&[u8],sender:SocketAddr)->bool{
        debug!("PING Message Event from {}- {} bytes",sender,msg.len());
        return true;
        }
    fn time_message(&mut self,msg:&[u8],sender:SocketAddr)->bool{
        debug!("TIME Message Event from {}- {} bytes",sender,msg.len());
        let tnow: u64 = time::precise_time_ns();
        debug!("Server Time is now {}",tnow);
        let mut wtr:Vec<u8>=vec![0;8];
        LittleEndian::write_u64(&mut wtr, tnow);
        let mut out = vec![0;10];
        out[0] =MSG_WORLD;
        out[1]=SR_TIME;
        out.append(&mut wtr);
        self.broadcast(&out[0..],sender,COND_OWNERONLY);
        return true;
    }
    fn ack_message(&mut self,msg:&[u8],sender:SocketAddr)->bool{
        return true;
    }

    fn broadcast(&mut self,msg:&[u8],sender:SocketAddr,rep_condition:u8)->bool{
        info!("Connections: {}",&self.connections.len());
        match rep_condition{
            COND_OWNERONLY=>{
                    self.socket.send_to(&msg[0..],&sender);
                (true)
            }
            COND_SKIPOWNER=>{
                debug!("Skipping send to {}",sender);
                for (id,client) in &self.connections{
                    if(client.addr != sender){
                        debug!("send to {}",client.addr);

                        self.socket.send_to(&msg[0..],&client.addr);
                    }
                }
                (true)
                
            }
            COND_NONE=>{
                for (id,client) in &self.connections{
                    self.socket.send_to(&msg[0..],&client.addr);
                }
                (true)
            }
            _=>{
                (true)
            }
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Client {
    pub instant: Instant,
    pub guid: u32,
    pub addr: SocketAddr,
}
trait Connections{

    fn add_connection(&mut self,id:u32, addr:SocketAddr)->bool;
    fn notify_new_connection(&mut self,id:u32)->bool;
}
impl Connections for Server{
    fn add_connection(&mut self,id:u32, addr:SocketAddr)->bool{
        use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut x = rng.gen::<u32>();
    for (uid,client) in self.connections.iter(){
            if client.addr==addr{
                x=client.guid;
            }   
    }
       let newClient = Client{
            instant: Instant::now(),
            guid: x,
            addr: addr
        };
        self.connections.insert(x,newClient);
        debug!("Adding connection: [{}] {}. Now {}",x,addr,self.connections.len());
        self.notify_new_connection(x);
        let msg_map = HashMap::<SocketAddr,&[u8]>::new();
        let mut existing:Vec<u32> = vec![];
         for (uid,client) in self.connections.iter(){
            existing.push(*uid);   
         }
        for uid in existing{
            if(x!=uid){
                 let mut msg = vec![MSG_WORLD,SR_JOINED];
                let mut wtr:Vec<u8>=vec![0;6];
                LittleEndian::write_u32(&mut wtr, uid);
//                wtr = format!("{:?} New connection",uid).into_bytes();
                msg.extend_from_slice(&wtr);
                 debug!("sending connection: [{}]",x);
     
                self.broadcast(&msg[0..],addr,COND_OWNERONLY);
            }else{
                let mut msg = vec![MSG_WORLD,SR_REGISTER];
                let mut wtr:Vec<u8>=vec![0;6];
                LittleEndian::write_u32(&mut wtr, uid);
//                wtr = format!("{:?} MY connection",uid).into_bytes();
                msg.extend_from_slice(&wtr);
                 debug!("sending connection: [{}]",x);
     
                self.broadcast(&msg[0..],addr,COND_OWNERONLY);
            }
        }
        return true
    }

    fn notify_new_connection(&mut self,id:u32)->bool{
        let addr=self.connections[&id].addr;
        let mut msg = vec![MSG_WORLD,SR_JOINED];
        let mut wtr:Vec<u8>=vec![0;6];
        LittleEndian::write_u32(&mut wtr, id);
        msg.extend_from_slice(&wtr);
//            msg = format!("{:?} MY connection",id).into_bytes();
        //self.broadcast(&msg[0..],addr,COND_SKIPOWNER);
        //debug!("Sending {:?} to {}",msg,COND_SKIPOWNER);
        return true
    }
}

impl Future for Server {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        loop {
            // First we check to see if there's a message we need to echo back.
            // If so then we try to send it back to the original source, waiting
            // until it's writable and we're able to do so.
            if let Some((size, peer)) = self.to_send {
                let mut cop = self.buf[..size].to_vec();
              //  let amt = try_nb!(self.socket.send_to(&self.buf[..size], &peer));
                let sr=self.parse_route(cop.clone(),peer);
                if(!sr){
                    info!("Echoed {} bytes to {}", size, peer);
                    self.socket.send_to(&cop[0..],&peer);
                }
                // sr.what();
                // let outb:&[u8]=Ok(sr).unwrap();
                // self.socket(send_to(cop.to))

                self.to_send = None;
            }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            self.to_send = Some(try_nb!(self.socket.recv_from(&mut self.buf)));
        }
    }

  
}

fn app()-> App<'static, 'static> {
    //! get args and info
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
    //! Main.
    env_logger::init();
    let matches = app().get_matches();
    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:8080");
    let addr = addr.parse::<SocketAddr>().unwrap();
    // Create the event loop that will drive this server, and also bind the
    // socket we'll be listening to.
    let mut l = Core::new().unwrap();
    let handle = l.handle();
    let socket = UdpSocket::bind(&addr, &handle).unwrap();
    info!("Listening on: {}", socket.local_addr().unwrap());

    // Next we'll create a future to spawn (the one we defined above) and then
    // we'll run the event loop by running the future.
    let mut  connections=HashMap::<u32,Client>::new();
    let mut server = Server{
        socket: socket,
        buf: vec![0; 1024],
        to_send: None,
        connections:connections
    };
    l.run(server).unwrap();
}
