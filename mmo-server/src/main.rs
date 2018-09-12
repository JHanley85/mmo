//! Echo mmo server
//!


#[macro_use]
extern crate clap;
extern crate futures;
extern crate tokio_core;
extern crate byteorder;
extern crate time;
#[macro_use] 
extern crate log;
extern crate uuid;
extern crate rand;

use clap::{Arg, App};
use std::io;

use std::hash::Hash;
use std::hash::Hasher;

use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use futures::{Future, stream, Stream, Sink};
use tokio_core::net::{UdpSocket, UdpCodec};
use tokio_core::reactor::Core;
use byteorder::{LittleEndian, ReadBytesExt,WriteBytesExt};


pub struct LineCodec;
/*
enum ENetAddress : uint8 {
	Ping = 0,
	World = 1,
	Avatar = 2,  //Hardcoded
	RigidBody = 3, //Hardcoded
	ByteArray = 4, //Hardcoded
	NetGuid = 5, //Hardcoded
	Float = 10,
	Int = 11,
	String = 12,
};
*/
static MSG_PING: u8=0;
static MSG_WORLD: u8=1;
static MSG_AVATAR: u8=2;
static MSG_BYTE_ARRAY: u8=4;
//Server requests

static SR_TIME:u8=0;
static SR_REGISTER:u8=1;
static SR_ACK:u8=2;
static SR_CALLBACK_UPDATE:u8=3;
static SR_RPC:u8=4;
static SR_PROPERTYREP:u8=5;
static SR_FUNCREP:u8=6;
//message Relevancy
static COND_INITIALONLY:u8=0; // - This property will only attempt to send on the initial bunch
static COND_OWNERONLY:u8=1; // - This property will only send to the actor's owner
static COND_SKIPOWNER:u8=2; // - This property send to every connection EXCEPT the owner
static COND_SIMULATEDONLY:u8=3; // - This property will only send to simulated actors
static COND_AUTONOMOUSONLY:u8=4; // - This property will only send to autonomous actors
static COND_SIMULATEDORPHYSICS:u8=5; //- This property will send to simulated OR bRepPhysics actors
static COND_INITIALOROWNER:u8=6; // - This property will send on the initial packet, or to the actors owner
static COND_CUSTOM:u8=7; // - This property has no particular condition, but wants the ability to toggle on/off via SetCustomIsActiveOverride
static COND_NONE:u8=8; // - This property will send to sender, and all listeners
static COND_SKIP:u8=0;
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


#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Client {
    pub instant: Instant,
    pub guid: u32,
    pub addr: SocketAddr,
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
    
    info!("Waiting for clients on: {}", addr);
    let listen_task = udp_socket_rx.fold((clients, udp_socket_tx), | (clients, udp_socket_tx), (client_socket, msg)| {
            let mut msg_dup = msg.clone();
            let mut rep_condition:u8 = COND_SKIPOWNER;
             
            if msg.len()==0  {
                rep_condition = COND_SKIP;
                ()

            }else{
                println!("IN [{}][{}]{} : {}  = {}",msg[0],msg[1],String::from_utf8_lossy(&[0u8]),String::from_utf8_lossy(&msg[0..]),msg.len());
                let channel: u8 = msg[0];

                if channel == MSG_WORLD{
                    // Special requests
                    let mut all_uuids: Vec<u32>=vec![];

                    for (_socket, client) in clients.iter() {
                        all_uuids.push(client.guid);
                    }

                    let (new_condition,newmsg,new_client)= parse_message(all_uuids,client_socket,msg_dup.clone());
                    msg_dup = newmsg.clone();
                    rep_condition=new_condition;
                    if new_client.guid!=0 {
                        if !clients.contains_key(&client_socket){
                            clients.insert(client_socket,Client { instant:new_client.instant,guid:new_client.guid,addr:client_socket});
                            println!("New client registered: {:?} : {}  Online: {:?}", client_socket,new_client.guid ,clients.len() + 1);
                        }
                    }
                    
                    println!("Changed rep condition: {} new message {} old message {}",rep_condition,msg_dup.len(),msg.clone().len());
                }

                if channel == MSG_PING { 
                        let mut rdr = std::io::Cursor::new(&msg[1..]);
                        let uid = rdr.read_u32::<LittleEndian>().unwrap();
                        rd_ping(client_socket,uid);
                        let new_client : Client =Client{ instant:Instant::now(),guid:uid,addr:client_socket};
                        clients.insert(client_socket, new_client);
                        rep_condition= COND_SKIP;
                }else if channel==MSG_AVATAR{

                }else if channel == MSG_BYTE_ARRAY {

                }

            }

            if !clients.contains_key(&client_socket){
                  ()
             }

            println!("Sender: {:?} Online: {:?}", client_socket, clients.len());
            // if !clients.contains_key(&client_socket){
            //     ()
            // }
            // if !clients.contains_key(&client_socket) {
            //     println!("Connected: {:?} Online: {:?}", client_socket, clients.len() + 1);
            //      if channel == MSG_PING{
            //         let mut rdr = std::io::Cursor::new(&msg[1..]);
            //         let uuid = rdr.read_u32::<LittleEndian>().unwrap();
            //         join(client_socket,uuid);
            //         let newClient : Client = register_client(client_socket,clients.values().collect());
            //         clients.insert(client_socket, newClient);

            //         println!("Connected: {:?} : {} -> {} Online: {:?}", client_socket,uuid,newClient.UUID ,clients.len() + 1);
            //      }
            // }
            // clients.insert(client_socket, Client { instant: Instant::now(),uuid:1u32 });

            let mut expired = Vec::new();
            for (client_socket, client) in clients.iter() {
                if client.instant.elapsed() > expiration {
                    expired.push(*client_socket);
                }
            }
            for client_socket in expired {
                clients.remove(&client_socket);
                println!("Expired: {:?} Online: {:?}", client_socket, clients.len());
            }
            let client_sockets: Vec<_> = clients.keys()
               .filter(|&&x| 
                (broadcast_filter(rep_condition,client_socket,x))
               )
                .map(|k| *k).collect();

            if msg_dup.len()==0{
                ()
            }
            println!("Out [{}]    [{}][{}]{} : {}  = {:?}",client_socket,msg_dup[0],msg_dup[1],String::from_utf8_lossy(&[0u8]),String::from_utf8_lossy(&msg_dup[0..]),msg);
            stream::iter_ok::<_, ()>(client_sockets)
            .fold(udp_socket_tx, move |udp_socket_tx, client_socket| {
                
                udp_socket_tx.send((client_socket, msg_dup.clone()))
                .map_err(|_| ())
            })
            .map(|udp_socket_tx| (clients, udp_socket_tx))
            .map_err(|_| Error::new(ErrorKind::Other, "broadcasting to clients"))
            
    });

    if let Err(err) = core.run(listen_task) {
        error!("{}", err);
    }
}

fn broadcast_filter(condition: u8,sender: SocketAddr, client:SocketAddr)->bool {
    let mut c = condition;
    if [COND_INITIALONLY,COND_SIMULATEDONLY,COND_AUTONOMOUSONLY,COND_SIMULATEDORPHYSICS,COND_INITIALOROWNER,COND_CUSTOM].contains(&condition){
        c=COND_NONE;
    }
    if c== COND_SKIPOWNER {
        return client != sender
    }else if c== COND_OWNERONLY {
        return client == sender
    } else if c == COND_NONE{
        //The rest are currently unimplemented; Spam.
        return true
    }else{
        return false
    }
}

fn rd_ping(client_socket: SocketAddr,uid: u32){
                 println!("Client: {} UUID: {}", client_socket, uid);
                 join(client_socket,uid);
}
 fn server_time()->Vec<u8>{
     //! send server time
    let tnow :u64 = time::precise_time_ns();
    println!("Server Time is now {}",tnow);
    let mut wtr = vec![];
    wtr.write_u64::<LittleEndian>(tnow).unwrap();
    let b=wtr;
    return b;
}

fn join(client_socket: SocketAddr,uid: u32){
    //! Handle logic when new client is detected.
    println!("Client: {} UUID: {}", client_socket, uid);
    println!("This is where we'd send initial state stuff.");
}

fn parse_message(clients: Vec<u32>,client_socket: SocketAddr,message: Vec<u8>)->(u8,Vec<u8>,Client){
    let mut outmessage:Vec<u8> = message.clone();
    let mut out_cond:u8=COND_NONE;
    let channel = message[0];
    let system:u8 = message[1];
    let newclient:Client= Client { instant:Instant::now(),guid: 0,addr:client_socket};
    if channel==MSG_WORLD {
            let mut rdr =std::io::Cursor::new(vec![message[2],message[3],message[4],message[5]]);
            let uid  = rdr.read_u32::<LittleEndian>().unwrap();
        if system==SR_REGISTER {
                println!("Recieved registration request");
                out_cond = COND_OWNERONLY;
                let newclient=register_client(clients,uid);
                outmessage = vec![MSG_WORLD,SR_REGISTER];
                let mut wtr = vec![];
                wtr.write_u32::<LittleEndian>(newclient.guid).unwrap();
                outmessage.append(&mut wtr);
                println!("Recieved SR_REGISTER request : uuid :{}-> {}",uid,newclient.guid);
                outmessage.append(&mut wtr);
            }else if system==SR_TIME {
                outmessage = vec![MSG_WORLD];
                outmessage[1]=SR_TIME;
                outmessage.append(&mut server_time());
                out_cond=COND_OWNERONLY;
                println!("Recieved SR_TIME request : uuid :{}",uid);
            
            }else   if system==SR_CALLBACK_UPDATE {
                outmessage = message;
                out_cond=COND_NONE;
                println!("Recieved SR_CALLBACK_UPDATE request : uuid :{}",uid);
           
            }else   if system==SR_PROPERTYREP {
                outmessage = message;
                out_cond=COND_NONE;
                println!("Recieved SR_PROPERTYREP request : uuid :{}",uid);
            
            }else   if system==SR_FUNCREP {
                outmessage = message;
                out_cond=COND_NONE;
                println!("Recieved SR_FUNCREP request : uuid :{}",uid);
            
            }else   if system==SR_ACK {
                outmessage = message;
                out_cond=COND_NONE;
                println!("Recieved SR_ACK request : uuid :{}",uid);
            }else   if system==SR_RPC {
                outmessage = message;
                out_cond=COND_SKIP;
                println!("Recieved SR_RPC request : uuid :{}",uid);
            
            
            }  else{
                outmessage = message;
                out_cond=COND_NONE;
                println!("Recieved _ request : uuid :{}",uid);
            }
    }

    return (out_cond,outmessage,newclient)
}
fn assign_uuid(clients: Vec<u32>,_requested_uuid:u32) ->  Vec<u8>{
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut x:u32 = _requested_uuid;
    while  clients.contains(&x) || x==0 {
            let mut s = std::collections::hash_map::DefaultHasher::new();
            uuid::Uuid::new_v4().hash(&mut s);
            x=((s.finish() + 10000) as u16) as u32 | 0b00000000000000100000000000000000;
    }
    x= rng.gen::<u32>();
    let mut wtr = vec![];
    wtr.write_u32::<LittleEndian>(x).unwrap();
    return wtr
}
fn register_client(clients: Vec<u32>,requested_uuid:u32)->Client {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    let mut rdr = std::io::Cursor::new(  assign_uuid(clients,requested_uuid));
    let uid: u32 = rdr.read_u32::<LittleEndian>().unwrap();
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    return Client { instant:Instant::now(),guid: uid,addr:socket}
}
