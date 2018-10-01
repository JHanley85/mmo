
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

#[macro_use]
extern crate clap;
use clap::{Arg, App};
extern crate futures;
#[macro_use]
extern crate tokio_core;
extern crate byteorder;
extern crate time;
extern crate rand;

use std::{io};
use std::net::SocketAddr;

use futures::{Future, Poll};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Core;

use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
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
const SR_SESSION_UPDATE:u8=11;
const SR_PLAYER_STATE_UPDATE:u8=12;
const SR_REGISTRATION_SPAWNED:u8=13;
const SR_REGISTER_PROPERTY:u8=14;
const SR_REQUEST_USERSTATE:u8=15;
const SR_REQUEST_OBJECTSTATE:u8=16;
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

#[derive(Eq, PartialEq, Debug,Clone)]
struct ObjectRep{
    oid:u32,
    parent_id:u32,
    bytes:Vec<u8>,
}

struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,
    connections:HashMap<u32,Client>,
    objects:HashMap<u32,ObjectRep>
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
        if data_source.len() < 1 {
            return 0;
        }else{
            return data_source[0];
        }
    }
    fn parse_route(&mut self,msg:Vec<u8>,peer:SocketAddr)->bool{
        let mut outmessage:Vec<u8> = msg.clone();
        let mut out_cond:u8=COND_NONE;
        if msg.len() < 2 {
            warn!("Rec'd small msg from {} {:?}",peer,&msg[0..]);
            return true;
        }
        let channel = msg[0];
        let system = msg[1];
        let senderid = self.get_sender_id(peer);
        debug!("C{} S{}, from {}",channel,system,senderid);
        let mut is_server_request = channel==MSG_WORLD;
        if channel==MSG_WORLD {
            match system {
                SR_REGISTER=>{
                    self.register_connection(&msg[2..],peer);
                        is_server_request=true;
                    ()
                }
                SR_TIME=>{
                    if senderid!=0 {
                        self.time_message(&msg[2..],peer);
                        is_server_request=true;
                    }
                    ()
                }
                SR_CALLBACK_UPDATE=>{
                    if senderid!=0 {
                        self.callback_message(&msg[2..],peer);
                        is_server_request=true;
                    }()
                }
                SR_PROPERTYREP=>{
                    if senderid!=0 {
                        self.property_message(&msg[0..],peer);
                        is_server_request=true;
                    }
                    ()
                }
                SR_FUNCREP=>{
                    if senderid!=0 {
                        self.function_message(&msg[2..],peer);
                        is_server_request=true;
                    }
                    ()
                }
                SR_ACK=>{
                    if senderid!=0 {
                        self.ack_message(&msg[2..],peer);
                        is_server_request=true;
                    }
                    ()
                }
                SR_RPC=>{
                    if senderid!=0 {
                        self.rpc_message(&msg[2..],peer);
                        is_server_request=true;
                    }
                    ()
                }
                SR_CLOSED=>{
                    if senderid!=0 {
                        self.closed_connection(&msg[2..],peer);
                        is_server_request=true;
                    }
                    ()
                }
                SR_REGISTRATION_SPAWNED=>{
                    if senderid!=0 {
                        self.register_object(&msg[2..],peer);
                        is_server_request=true;
                    }
                    ()
                }
                SR_STATE=>{
                        is_server_request=true;
                    ()
                }
                SR_SESSION_UPDATE=>{
                        is_server_request=true;
                    ()
                }
                SR_AUTHORITY=>{
                        is_server_request=true;
                    ()
                }
                SR_PLAYER_STATE_UPDATE=>{
                        is_server_request=true;
                    ()
                }
                SR_REGISTER_PROPERTY=>{
                    is_server_request=true;
                    self.register_property(&msg[2..],peer);
                    ()
                }
                SR_REQUEST_USERSTATE=>{
                    is_server_request=true;
                    self.userstate(&msg[2..],peer);
                    ()
                }
                SR_REQUEST_OBJECTSTATE=>{
                    is_server_request=true;
                    self.objectstate(&msg[2..],peer);
                    ()
                }
                _=>{
                    is_server_request=false;
                    ()
                }
            }
        }else if channel==MSG_AVATAR || channel==MSG_PING || channel==MSG_BYTE_ARRAY{
            is_server_request=false;
        }
        return is_server_request;
    }
    fn closed_connection(&mut self,msg:&[u8],sender:SocketAddr)->bool{
        debug!("CLOSE Message Event from {}- {} bytes",sender,msg.len());
       let data_source : &Vec<u32>  =  &self.connections.iter()
        .filter(|&(k,v)| v.addr == sender).map(|(k,v)| *k).collect();
        for uid in data_source.iter(){
            &self.connections.remove(uid);
            let mut msg = vec![MSG_WORLD,SR_CLOSED];
            let mut wtr:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut wtr, *uid);
            msg.extend_from_slice(&wtr);
            info!("CLOSE Message Event from {} -> {} ",sender,uid);
            self.broadcast(&msg[0..],sender,COND_SKIPOWNER);
        }
       
        return true;
    }
    fn register_connection(&mut self,msg:&[u8],sender:SocketAddr)->bool{
          let s = String::from_utf8_lossy(&msg[4..]);
        debug!("REGISTER Message Event from {}- {} bytes {}",sender,msg.len(),s);
        
        self.add_connection(&msg[4..],sender);
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
        let mut cond = rep_condition;
        if rep_condition==COND_INITIALONLY || 
            rep_condition==COND_SIMULATEDONLY || 
            rep_condition==COND_CUSTOM ||
            rep_condition==COND_INITIALOROWNER ||
            rep_condition==COND_AUTONOMOUSONLY ||
            rep_condition==COND_SIMULATEDORPHYSICS ||
            rep_condition==COND_SKIP ||
            rep_condition==COND_SKIPOWNER{
                cond=COND_NONE;
            }
        match cond{
            COND_OWNERONLY=>{
                    drop(self.socket.send_to(&msg[0..],&sender));
                (true)
            }
            COND_SKIPOWNER=>{
                debug!("Skipping send to {}",sender);
                for (id,client) in &self.connections{
                    if client.addr != sender {
                        debug!("send to {}",client.addr);

                        drop(self.socket.send_to(&msg[0..],&client.addr));
                    }
                }
                (true)
                
            }
            COND_NONE=>{
                for (id,client) in &self.connections{
                    drop(self.socket.send_to(&msg[0..],&client.addr));
                }
                (true)
            }
            _=>{
                (true)
            }
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug,Clone)]
pub struct Client {
    pub instant: Instant,
    pub guid: u32,
    pub addr: SocketAddr,
    pub settings:Vec<u8>,
    pub last_message:Instant
}

trait Connections{
    fn add_connection(&mut self,msg:&[u8], addr:SocketAddr)->bool;
    fn notify_new_connection(&mut self,id:u32)->bool;
    fn userstate(&mut self,msg:&[u8],addr:SocketAddr)->bool;
}
trait Objects{
    fn register_object(&mut self,msg:&[u8],addr:SocketAddr)->bool;
    fn notify_registered_object(&mut self,id:u32)->bool;
    fn register_property(&mut self,msg:&[u8],addr:SocketAddr)->bool;
    fn objectstate(&mut self,msg:&[u8],addr:SocketAddr)->bool;
}
impl Connections for Server{
    fn add_connection(&mut self,msg:&[u8], addr:SocketAddr)->bool{
        use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut x = rng.gen::<u32>();
        for (uid,client) in self.connections.iter(){
                if client.addr==addr{
                    x=client.guid;
                }   
        }
       let new_client = Client{
            instant: Instant::now(),
            guid: x,
            addr: addr,
            settings: msg.to_vec(),
            last_message:Instant::now(),
        };
        if !self.connections.contains_key(&x){
            self.connections.insert(x,new_client);
        }
        debug!("Adding connection: [{}] {}. Now {}",x,addr,self.connections.len());
        let msg_map = HashMap::<SocketAddr,&[u8]>::new();
        let mut existing:Vec<u32> = vec![];
         let mut clients:Vec<Client> = vec![];

        for (uid,client) in self.connections.iter(){
            existing.push(*uid);   
            clients.push(client.clone());
         }
        for client in clients{
            
            if x!=client.guid {
                let mut out = vec![MSG_WORLD,SR_JOINED];
                let mut wtr:Vec<u8>=vec![0;4];
                LittleEndian::write_u32(&mut wtr, client.guid);
                out.extend_from_slice(&wtr);
                out.extend_from_slice(&client.settings[0..]);
                 debug!("sending connection: [{}]",x);
     
                self.broadcast(&out[0..],addr,COND_OWNERONLY);
            }else{
                let mut out = vec![MSG_WORLD,SR_REGISTER];
                let mut wtr:Vec<u8>=vec![0;4];
                LittleEndian::write_u32(&mut wtr, client.guid);
                out.extend_from_slice(&wtr);
                out.extend_from_slice(&client.settings[0..]); 
                let s = String::from_utf8_lossy(&client.settings[0..]);
                   println!("result: {} {}", s,&client.settings[0..].len());
                 debug!("sending connection: [{}]",x);
     
                self.broadcast(&out[0..],addr,COND_OWNERONLY);
            }
        }
        return true
    }

    fn notify_new_connection(&mut self,id:u32)->bool{
        let addr=self.connections[&id].addr;
        let mut msg = vec![MSG_WORLD,SR_JOINED];
        let mut wtr:Vec<u8>=vec![0;4];
        LittleEndian::write_u32(&mut wtr, id);
        msg.extend_from_slice(&wtr);
        self.broadcast(&msg[0..],addr,COND_SKIPOWNER);
        debug!("Sending {:?} to {}",msg,COND_SKIPOWNER);
        return true
    }
    fn userstate(&mut self,msg:&[u8],addr:SocketAddr)->bool{
        let mut oid:u32 =LittleEndian::read_u32(&msg[0 .. 4]);
        let object:Client= self.connections[&oid].clone();
        self.broadcast(&object.settings[0..],addr,COND_OWNERONLY);
        return true;
    }
}
impl Objects for Server{
     fn register_property(&mut self,msg:&[u8],addr:SocketAddr)->bool{
        use rand::Rng;
        debug!("PROPERTY_REGISTRATION Message Event from {}- {} bytes",addr,msg.len());
        let mut sender:u32 =LittleEndian::read_u32(&mut &msg[0..4]);
        let mut oid:u32 =LittleEndian::read_u32(&mut &msg[4..8]);
        let mut pid:u32 =LittleEndian::read_u32(&mut &msg[8..12]);
        let mut payload = &msg[12..];
        let mut rng = rand::thread_rng();
        let mut new_oid = rng.gen::<u32>();
        while self.objects.contains_key(&new_oid){
            new_oid = rng.gen::<u32>();
        }
        debug!("[{}] {} {}=>{} payload {:?}",sender, oid,pid,new_oid, &payload[0..]);

        let mut client = self.connections.values().cloned().filter(|ref c| c.addr==addr).next().clone().unwrap();
        let new_prop = ObjectRep{
            parent_id: oid,
            bytes:payload.to_vec(),
            oid:new_oid,
        };
        self.objects.insert(new_oid,new_prop);
         
        let mut out_sender = vec![MSG_WORLD,SR_REGISTER_PROPERTY];
        //sender uid
        let mut uid:Vec<u8>=vec![0;4];
        LittleEndian::write_u32(&mut uid, client.guid);
        out_sender.extend_from_slice(&uid);
        //object id
        let mut orig_id:Vec<u8>=vec![0;4];
        LittleEndian::write_u32(&mut orig_id, oid);
        out_sender.extend_from_slice(&orig_id);
        // new property id
        let mut new_id:Vec<u8>=vec![0;4];
        LittleEndian::write_u32(&mut new_id, new_oid);
        out_sender.extend_from_slice(&new_id);
        // property payload - being name.
        out_sender.extend_from_slice(&payload[0..]);
        self.broadcast(&out_sender[0..],addr,COND_NONE);

        return true;
     }
     fn register_object(&mut self,msg:&[u8],addr:SocketAddr)->bool{
        use rand::Rng;
        debug!("OBJECT_REGISTRATION Message Event from {}- {} bytes ={:?}",addr,&msg[0 .. 3].len(),&msg);

        let mut uid:u32 =LittleEndian::read_u32(&msg[0 .. 4]);
        let mut oid:u32 =LittleEndian::read_u32(&msg[4 .. 8]);
        let mut rng = rand::thread_rng();
        let mut new_oid = rng.gen::<u32>();
        while self.objects.contains_key(&new_oid){
            new_oid = rng.gen::<u32>();
        }
        let mut payload:Vec<u8>=msg[8..].to_vec();
        debug!("[{}] {}=>{} payload {:?}",uid, oid,new_oid, &payload[0..]);
 
        let mut new_object = ObjectRep{
            parent_id:0,
            oid:new_oid,
            bytes:payload.clone()
        };
        self.objects.insert(new_oid,new_object);
        self.notify_registered_object(oid);
        let mut clients=self.connections.values().cloned()
            .filter(|ref v| v.addr==addr)
            .next().clone();
        let client = clients.unwrap();
       
            let mut out_sender = vec![MSG_WORLD,SR_REGISTRATION_SPAWNED];
            let mut uid:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut uid, client.guid);
            out_sender.extend_from_slice(&uid);
            let mut orig_id:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut orig_id, oid);
            out_sender.extend_from_slice(&orig_id);
            let mut new_id:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut new_id, new_oid);
            out_sender.extend_from_slice(&new_id);
            out_sender.extend_from_slice(&payload[0..]);
            debug!("Sending {:?}",&out_sender[0..]);
            self.broadcast(&out_sender[0..],addr,COND_NONE);


         return true;

     }
    fn notify_registered_object(&mut self,id:u32)->bool{
        return false;
    }
     fn objectstate(&mut self,msg:&[u8],addr:SocketAddr)->bool{
        let mut oid:u32 =LittleEndian::read_u32(&msg[0 .. 4]);
        let object: ObjectRep = self.objects[&oid].clone();

        self.broadcast(&object.bytes[0..],addr,COND_OWNERONLY);
        return true;
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
                if !sr {
                    info!("Echoed [{}][{}]{} bytes to {}",&cop[0],&cop[1], size, peer);
                    drop(self.socket.send_to(&cop[0..],&peer));
                }
                // sr.what();
                // let outb:&[u8]=Ok(sr).unwrap();
                // self.socket(send_to(cop.to))

                self.to_send = None;
            }
            info!("Connections: {}",self.connections.len());
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
    let mut objects=HashMap::<u32,ObjectRep>::new();
    l.run(Server{
        socket: socket,
        connections:connections,
        objects:objects,
        buf: vec![0; 1024],
        to_send: None,
       }).unwrap();
}
