//! An UDP echo server that just sends back everything that it receives.
//!
//! If you're on Unix you can test this out by in one terminal executing:
//!
//!     cargo run --example echo-udp
//!
//! and in another terminal you can run:
//!
//!     cargo run --example connect -- --udp 127.0.0.1:8080
//!
//! Each line you type in to the `nc` terminal should be echo'd back to you!

#![feature(async_await)]
#![warn(rust_2018_idioms)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_extern_crates)]

#[macro_use]
extern crate log;
extern crate env_logger;

use std::error::Error;
use std::net::SocketAddr;
use std::{ io};
// // use std::time::{Instant, Duration};
use tokio;
use tokio::net::UdpSocket;
mod server;
use server::definitions::{ClientRoute,ServerRoute};

use server::objects::{Client,ObjectRep};
use std::fmt::{Formatter,Display};

struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,
    clients: Vec<String>,
    objects: Vec<String>
}

// use std::{thread};
// use std::io::prelude::*;

extern crate byteorder;
use byteorder::{ByteOrder, LittleEndian,WriteBytesExt};
use std::collections::HashMap;
extern crate fred;
use fred::RedisClient;
use fred::owned::RedisClientOwned;
use fred::types::*;
use futures::Future;
type ServerResult = Option<String>;
type EchoResult = Option<Vec<u8>>;
impl Server {
    // async fn get_all_clients()->Option<Vec<Client>>{

    // }
    async fn parse_client_route(msg:Vec<u8>)->Result<(ClientRoute,ServerRoute),()>{
        use rand::Rng;
        let mut rng = rand::thread_rng();
        //let i=rng.gen_range(1,100);
        //  thread::sleep(Duration::from_millis(i));
        if msg[0]== ClientRoute::World as u8{
            return Ok((ClientRoute::from(msg[0]), ServerRoute::from(msg[1])))
        }
        return Err(())
    }
    async fn world_register(msg:Vec<u8>,addr:SocketAddr)->ServerResult{
        let pbytes = addr.port().to_be_bytes(); 
        let mut out = vec![0,0,pbytes[0],pbytes[1]];
        let mut oid:u32 =LittleEndian::read_u32(&out);
        let client = Client::new(oid,addr,msg[4..].to_vec());

        let mut cache = memcache::Client::connect("memcache://memcache:11211?timeout=10&tcp_nodelay=true").unwrap();
        let mut _key = format!("client_{}",oid);

        cache.set(&_key,client.clone().serialized().unwrap(),0);
        println!("Registered CLIENT: {}",client.clone());
        Some(_key)
    }
    async fn register_property(msg:Vec<u8>,addr:SocketAddr)->ServerResult{
        use rand::Rng;
        let sender:u32 = LittleEndian::read_u32(&mut &msg[0..4]);
        let parent_id:u32 = LittleEndian::read_u32(&mut &msg[4..8]);
        let id:u32 = LittleEndian::read_u32(&mut &msg[8..12]);
        let data:Vec<u8> = msg[12..].to_vec();
        let mut rng = rand::thread_rng();
        let new_id = rng.gen::<u32>();
        let property = ObjectRep::new(
            id,
            new_id,
            parent_id,
            data,
            sender
        );
        let mut cache = memcache::Client::connect("memcache://memcache:11211?timeout=10&tcp_nodelay=true").unwrap();
        let mut _key = format!("property_{}",new_id);

        cache.set(&_key,property.clone().serialized().unwrap(),0);
        println!("Registered PROPERTY: {}",property.clone());
        Some(_key)
    }
    async fn replicate_property(msg:Vec<u8>,addr:SocketAddr)->EchoResult{
        let owner = LittleEndian::read_u32(&mut &msg[2..6]);
        let id = LittleEndian::read_u32(&mut &msg[6..10]);
        let value = &msg[11..];
        Some(msg[0..].to_vec())
    }
    async fn world_register_object(msg:Vec<u8>,addr:SocketAddr)->ServerResult{
        use rand::Rng;
        let mut client_id:u32 =LittleEndian::read_u32(&msg[0..4]);
        let mut rng = rand::thread_rng();
        let mut new_id = rng.gen::<u32>();
        
        let mut obj_id:u32 =LittleEndian::read_u32(&msg[4 .. 8]);
        let payload = &msg[8..];
        // if client_id==0{
		// 	let i:Vec<u32>=self.connections.values().cloned()
        //     .filter(|v| v.addr == addr).map(|v| v.guid).collect();
		// 	client_id=i[0];
		// }
        let mut original_id:u32 =LittleEndian::read_u32(&msg[4 .. 8]);
        let parent_id:u32=0;
        let mut object:ObjectRep= ObjectRep::new(obj_id,new_id,parent_id,payload.to_vec(),client_id);
        let mut cache = memcache::Client::connect("memcache://memcache:11211?timeout=10&tcp_nodelay=true").unwrap();
        let mut _key = format!("object_{}",object.id);
            
        cache.set(&_key,object.clone().serialized().unwrap(),0);
        println!("Registered OBJECT {}",object.clone());
        Some(_key)
    }
    
    async fn run(self) -> Result<(), io::Error> {
          let Server {
              mut socket,
              mut buf,
              mut to_send,
              mut clients,
              mut objects
         } = self;
       let mut client = memcache::Client::connect("memcache://memcache:11211?timeout=10&tcp_nodelay=true").unwrap();

        // flush the database
        client.flush().unwrap();

        // set a string value
        client.set("foo", "bar", 0).unwrap();

        // retrieve from memcached:
        let value: Option<String> = client.get("foo").unwrap();
        let vcopy = value.clone();
        assert_eq!(value, Some(String::from("bar")));
        assert_eq!(value.unwrap(), "bar");
        println!("{:?}",vcopy);
                loop {
                     let stats = client.stats().unwrap();
                     let mut clients = clients;
                     println!("Stats: {:?}",stats);
                    // First we check to see if there's a message we need to echo back.
                    // If so then we try to send it back to the original source, waiting
                    // until it's writable and we're able to do so.
                    if let Some((size, peer)) = to_send {
                       // let amt = socket.send_to(&buf[..size], &peer).await?;
                        let result: std::collections::HashMap<String, String> = client.gets(clients.clone().iter().map(|s| &**s).collect()).unwrap(); 
                        println!("Clients {:?}",result);
                        let bufcopy = buf.clone();
                        let addr = peer.clone();
                    
                        let c=tokio::spawn(async move{
                            let mut clients = clients;
                            let bufcopy = bufcopy.clone();
                            let addr = addr.clone();
                            let cr= Server::parse_client_route(bufcopy.clone().to_vec()).await.unwrap();
                            match cr {
                                (ClientRoute::World,ServerRoute::Register)=>{
                                    println!("World Register player");
                                    match Server::world_register(bufcopy.to_vec(),addr).await {
                                        Some(client_key)=>{
                                            println!("client key {}",&client_key);
                                            clients.push(client_key);
                                          //  clients.insert(client.guid,client);
                                            ()
                                        },
                                        None=>{
                                        ()
                                        }
                                    }
                                    ()
                                },
                                (ClientRoute::World,ServerRoute::RegistrationSpawned)=>{
                                    println!("register object");
                                    match Server::world_register_object(bufcopy.to_vec(),addr).await {
                                        Some(object_key)=>{
                                            println!("object key {}",&object_key);
                                            objects.push(object_key);
                                        },
                                        None=>{

                                        }
                                    }
                                },
                                (ClientRoute::World,ServerRoute::PropertyRep)=>{
                                    println!("property replication");
                                    match Server::replicate_property(bufcopy.to_vec(),addr).await{
                                        Some(echo)=>{
                                            println!("object property update {:?}",&echo);
                                        
                                        },
                                        None=>{

                                        }
                                    }
                                },
                                (ClientRoute::World,ServerRoute::RegisterProperty)=>{
                                    println!("register property");
                                    match Server::register_property(bufcopy.to_vec(),addr).await{
                                        Some(object_key)=>{
                                            println!("property registration {}",&object_key);
                                            objects.push(object_key);
                                        },
                                        None=>{

                                        }
                                    }
                                }
                                _=>()
                            }
                           // println!("Echoed [{:?}] {}/{} bytes to {}", cr,amt, size, peer);
                        });
                    }

                    // If we're here then `to_send` is `None`, so we take a look for the
                    // next message we're going to echo back.
                    to_send = Some(socket.recv_from(&mut buf).await?);
                }
                
            }
        }
use clap::{Arg, App};
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let matches = app().get_matches();
    let addr = matches.value_of("addr").unwrap_or("127.0.0.1:8080");
    // let addr = addr.parse::<SocketAddr>().unwrap();
    // let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>()?;

    let socket = UdpSocket::bind(&addr)?;
    println!("Listening on: {}", socket.local_addr()?);

    let server = Server {
        socket,
        buf: vec![0; 65507],
        to_send: None,
        clients:vec![],
        objects:vec![]
    };

    // This starts the server task.
    server.run().await?;

    Ok(())
}