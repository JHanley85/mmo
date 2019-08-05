
use std::{io as stdio};
use std::net::SocketAddr;
use std::string::String;
use std::time::{SystemTime, UNIX_EPOCH};
use futures::{Future, Poll,Stream,Async,Sink};
extern crate tokio;

use tokio::io;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::net::UdpSocket;
use tokio::prelude::stream::*;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use byteorder::{ByteOrder, LittleEndian,WriteBytesExt};

use tokio::reactor::Handle;
use std::fmt::{Formatter,Error,Display};

pub mod objects;
use self::objects::{Client,ObjectRep};
pub mod definitions;
use self::definitions::{ServerRoute,ClientRoute,Relevancy};

mod rust_test;
// use rust_test::{write_to_file};


const MSG_PING:u8 = ClientRoute::Ping as u8;
const MSG_WORLD:u8 = ClientRoute::World as u8;
const MSG_AVATAR:u8 = ClientRoute::Avatar as u8;
const MSG_BYTE_ARRAY:u8 = ClientRoute::ByteArray as u8;

const SR_TIME:u8=ServerRoute::Time as u8;
const SR_REGISTER:u8=ServerRoute::Register as u8;
const SR_ACK:u8=ServerRoute::Ack as u8;
const SR_CALLBACK_UPDATE:u8=ServerRoute::CallbackUpdate as u8;
const SR_RPC:u8=ServerRoute::Rpc as u8;
const SR_PROPERTYREP:u8=ServerRoute::PropertyRep as u8;
const SR_FUNCREP:u8=ServerRoute::FunctionRep as u8;
const SR_AUTHORITY:u8=ServerRoute::Authority as u8;
const SR_STATE:u8=ServerRoute::State as u8;
const SR_JOINED:u8=ServerRoute::Joined as u8;
const SR_CLOSED:u8=ServerRoute::Closed as u8;
const SR_SESSION_UPDATE:u8=ServerRoute::SessionUpdate as u8;
const SR_PLAYER_STATE_UPDATE:u8=ServerRoute::PlayerStateUpdate as u8;
const SR_REGISTRATION_SPAWNED:u8=ServerRoute::RegistrationSpawned as u8;
const SR_REGISTER_PROPERTY:u8=ServerRoute::RegisterProperty as u8;
const SR_REQUEST_USERSTATE:u8=ServerRoute::RequestUserState as u8;
const SR_REQUEST_OBJECTSTATE:u8=ServerRoute::RequestObjectState as u8;
const SR_REQUEST_PROPERTYSTATE:u8=ServerRoute::RequestPropertyState as u8;
const SR_REJOIN:u8=ServerRoute::Rejoin as u8;
const SR_UNREGISTER_OBJECT:u8=ServerRoute::UnregisterObject as u8;
const SR_VOICE:u8=ServerRoute::Voice as u8;

const COND_INITIALONLY:u8 = Relevancy::InitialOnly as u8;
const COND_OWNERONLY:u8= Relevancy::OwnerOnly as u8;
const COND_SKIPOWNER:u8= Relevancy::SkipOwner as u8;
const COND_SIMULATEDONLY:u8= Relevancy::SimulatedOnly as u8;
const COND_AUTONOMOUSONLY:u8= Relevancy::AutonomousOnly as u8;
const COND_SIMULATEDORPHYSICS:u8= Relevancy::SimulatedOrPhysics as u8;
const COND_INITIALOROWNER:u8= Relevancy::InitialOrOwner as u8;
const COND_CUSTOM:u8= Relevancy::Custom as u8;
const COND_NONE:u8 = Relevancy::None as u8;
const COND_SKIP:u8= Relevancy::Skip as u8;




// pub struct UeBytes;
// impl UdpCodec for UeBytes{
//     type In =(SocketAddr,Vec<u8>);
//     type Out = (SocketAddr,Vec<u8>);

//     fn decode(&mut self,addr:&SocketAddr, buf: &[u8])->io::Result<Self.In>{
//        Ok((*addr,buf.to_vec()))
//     }
//   fn encode(&mut self, (addr, buf): Self::Out, into: &mut Vec<u8>) -> SocketAddr {       into.extenc(buf);
//         addr
//     }
// }





pub struct Server {
    addr:SocketAddr,
    socket: UdpSocket,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,
    connections:HashMap<u32,Client>,
    objects:HashMap<u32,ObjectRep>
}
impl Server{
    pub fn new(addr:SocketAddr)-> Server {
        let s = UdpSocket::bind(&addr).expect("failed to bind socket");
        let server:Server = Server{
            addr:addr,
            socket:s,
            buf: vec![0;65507],
            connections: HashMap::<u32,Client>::new(),
            objects: HashMap::<u32,ObjectRep>::new(),
            to_send: None,
        };
       

        println!("RedpillVR Server {}", server);
        return server;
    }
//    pub fn connect(&mut self)-> Box<dyn Stream<Item = Vec<u8>, Error = io::Error>>{
        
        // Box::new(stream.filter_map(move |(src,chunk)|{
        //     if src==self.addr{
        //         Some(chunk.into())
        //     }else{
        //         None
        //     }
        // }))
  //  }
    // pub fn run(&mut self, core: &mut core){
    //     let handle = core.handle();
    //     let socket = UdpSocket
    // }
}
impl Display for Server{
    fn fmt(&self,fmt: &mut Formatter) -> Result<(),Error>{
        write!(fmt,"Listening on: {}; Buffer: {}; Connections[{}]; Objects [{}];"
        ,self.socket.local_addr().unwrap()
        ,self.buf.len()
        ,self.connections.len()
        ,self.objects.len()
        )
    }
}
impl Drop for Server{
    fn drop(&mut self){
        println!("Dropping server {}",self);
    }
}



pub trait Router{
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
	fn update_client(&mut self,peer:SocketAddr)->bool;
	fn close_stale_connections(&mut self)->bool;
	fn close_connection(&mut self,uid:&u32)->bool;
}

use std::borrow::BorrowMut;
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
            
        let mut peer_string: String = peer.to_string().to_owned();

        if channel==MSG_WORLD {
            match system {
                SR_REGISTER=>{
                    self.register_connection(&msg[2..],peer);
                        is_server_request=true;
                        let borrowed_string: &str = ".register.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                    ()
                }
                SR_TIME=>{
                    if senderid!=0 {
                        self.time_message(&msg[2..],peer);
                        is_server_request=true;
                        let borrowed_string: &str = ".time.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                    }
                    ()
                }
                SR_CALLBACK_UPDATE=>{
                    if senderid!=0 {
                        self.callback_message(&msg[2..],peer);
                        is_server_request=true;
                        let borrowed_string: &str = ".callback.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                    }()
                }
                SR_PROPERTYREP=>{
                    if senderid==0 {
                        self.property_message(&msg[0..],peer);
                        is_server_request=true;
                        let borrowed_string: &str = ".prop_rep.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                    }
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
                        let borrowed_string: &str = ".func_rep.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                    }
                    ()
                }
                SR_ACK=>{
                    if senderid!=0 {
                        self.ack_message(&msg[2..],peer);
                        is_server_request=true;
                        let borrowed_string: &str = ".ack.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                    }
                    ()
                }
                SR_RPC=>{
                    if senderid!=0 {
                        self.rpc_message(&msg[2..],peer);
                        is_server_request=true;
                        let borrowed_string: &str = ".rpc.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                    }
                    ()
                }
                SR_CLOSED=>{
                    if senderid!=0 {
                        self.closed_connection(&msg[2..],peer);
                        let borrowed_string: &str = ".closed.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                        is_server_request=true;
                    }
                    ()
                }
                SR_REGISTRATION_SPAWNED=>{
                    if senderid!=0 {
                        self.register_object(&msg[2..],peer);
                        is_server_request=true;
                        let borrowed_string: &str = ".reg_spawned.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
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
                        let borrowed_string: &str = ".reg_property.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                    ()
                }
                SR_REQUEST_USERSTATE=>{
                    is_server_request=true;
                        let borrowed_string: &str = ".user_state.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                    self.userstate(&msg[2..],peer);
                    ()
                }
                SR_REQUEST_OBJECTSTATE=>{
                    is_server_request=true;
                        let borrowed_string: &str = ".obj_state.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                    self.objectstate(&msg[2..],peer);
                    ()
                }
				SR_REQUEST_PROPERTYSTATE=>{
                    is_server_request=true;
                    self.propertystate(&msg[2..],peer);
                        let borrowed_string: &str = ".prop_state.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                    ()
                }
                SR_UNREGISTER_OBJECT=>{
                    ()
                }
                SR_VOICE=>{
                    is_server_request=true;
                        let borrowed_string: &str = ".voice.req";
                        peer_string.push_str(borrowed_string);
                        drop(rust_test::write_to_file(&msg[0..],peer_string));
                    self.voicedata(&msg[0..],peer);
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
        info!("CLOSE Message Event from {}- {} bytes",sender,msg.len());
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
        let mut id = self.get_sender_id(sender);
        let mut sid=LittleEndian::read_u32(&msg[2 .. 6]);;
        if id==0 {
             id=LittleEndian::read_u32(&msg[2 .. 6]);
             let mut result=self.connections.get_mut(&id);
             match result{
                 Some(client)=>client.addr=sender,
                 None=>println!("Unknown guid {:?}",&id)
             }
        }
        //  let mut uid:u32 =LittleEndian::read_u32(&msg[0 .. 4]);

        debug!("PROPERTY Message Event from [{}][{}] {}- {} bytes",id,sid,sender,msg.len());
        self.broadcast(&msg[0..],sender,COND_SKIPOWNER);
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
        let mut cond = rep_condition;
        if rep_condition==COND_INITIALONLY || 
            rep_condition==COND_SIMULATEDONLY || 
            rep_condition==COND_CUSTOM ||
            rep_condition==COND_INITIALOROWNER ||
            rep_condition==COND_AUTONOMOUSONLY ||
            rep_condition==COND_SIMULATEDORPHYSICS ||
            rep_condition==COND_SKIP {
                cond=COND_NONE;
            }
         //   debug!("Sending {} bytes, {:?}",msg.len(),&msg[0..]);
        match cond{
            COND_OWNERONLY=>{
                self.socket.poll_send_to(&msg[0..],&sender);
                (true)
            }
            COND_SKIPOWNER=>{
                //info!("Skipping send to {}",sender);
                for (id,client) in &self.connections{
                    if client.addr != sender {
                        //info!("send to {}",client.addr);

                        drop(self.socket.poll_send_to(&msg[0..],&client.addr));
                    }
                }
                (true)
                
            }
            COND_NONE=>{
                for (id,client) in &self.connections{
                    drop(self.socket.poll_send_to(&msg[0..],&client.addr));
                }
                (true)
            }
            _=>{
                (true)
            }
        }
    }
	
	fn update_client(&mut self,peer:SocketAddr)->bool{
		for client in self.connections.values_mut(){
			if  client.is_same(peer) {
					info!("Updating Client");
					return client.update();
			}
		}
		 return false;
	}
	fn close_connection(&mut self,uid:&u32)->bool{
		info!("Closing connection {}",uid);
		&self.connections.remove(&uid);
		return true;
	}
	fn close_stale_connections(&mut self)->bool{
		let mut stale: Vec<u32>=Vec::<u32>::new();

		for (id,client) in &self.connections{
			if client.last_message.elapsed().as_secs() > 240{
				stale.push(*id);
			}
		}
		for id in stale{
			self.close_connection(&id);
            let stale_objects:Vec<u32> = self.objects.iter().filter(|(k,v)| v.owner==id).map(|(k,v)| *k).collect();
            for oid in stale_objects{
                self.unregister_object(oid);
            }
		}
        
		return true;
	}
}




pub trait Connections{
    fn add_connection(&mut self,msg:&[u8], addr:SocketAddr)->bool;
    fn notify_new_connection(&mut self,id:u32)->bool;
    fn userstate(&mut self,msg:&[u8],addr:SocketAddr)->bool;
    fn rejoin(&mut self, client:Client)->bool;
    fn create_uid(&mut self, addr: SocketAddr)->u32;
}
pub trait Objects{
    fn register_object(&mut self,msg:&[u8],addr:SocketAddr)->bool;
    fn unregister_object(&mut self,id:u32)->bool;
    fn notify_registered_object(&mut self,id:u32)->bool;
    fn register_property(&mut self,msg:&[u8],addr:SocketAddr)->bool;
    fn objectstate(&mut self,msg:&[u8],addr:SocketAddr)->bool;
    fn propertystate(&mut self,msg:&[u8],addr:SocketAddr)->bool;
}
pub trait Voice{
    fn voicedata(&mut self,msg:&[u8],addr:SocketAddr)->bool;
}


impl Voice for Server{
    fn voicedata(&mut self,msg:&[u8],addr:SocketAddr)->bool{
       return self.broadcast(&msg[0..],addr,COND_NONE);
    }
}


impl Connections for Server{
    fn rejoin(&mut self,client:Client)->bool{
        info!("Client rejoining: {} {}",client.guid,client.addr);
        let mut out = vec![MSG_WORLD,SR_REJOIN];
        let mut wtr:Vec<u8>=vec![0;4];
        LittleEndian::write_u32(&mut wtr, client.guid);
        out.extend_from_slice(&wtr);
        out.extend_from_slice(&client.settings[0..]); 
        let s = String::from_utf8_lossy(&client.settings[0..]);
        println!("result: {} {}", s,&client.settings[0..].len());
        debug!("sending reregisgtered connection: [{}]",client.guid);
        self.broadcast(&out[0..],client.addr,COND_OWNERONLY);
        return true;
    }
    fn create_uid(&mut self,addr: SocketAddr)->u32{

        let pbytes = addr.port().to_be_bytes() ; 
        println!("{:?}",pbytes);
        let mut out = vec![0,0,pbytes[0],pbytes[1]];
        let mut oid:u32 =LittleEndian::read_u32(&out);
      println!("Post");
        return oid;
    }

    fn add_connection(&mut self,msg:&[u8], addr:SocketAddr)->bool{
        use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

        use rand::Rng;
        let mut rng = rand::thread_rng();
        println!("One");
        let mut x = self.create_uid(addr);
        let mut preexisting = false;
        println!("Pre build");
        let new_client:Client = Client::new(x,addr,msg.to_vec());
        println!("Two");
        self.connections.insert(x,new_client.clone());
        info!("Adding connection: [{}] {}. Now {}",x,addr,self.connections.len());
        
        for (port32,client) in &self.connections.clone(){
    
            let mut pbytes:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut pbytes, *port32);
            let port = LittleEndian::read_u16(&pbytes);
            if x!=port.into() {
                let mut out = vec![MSG_WORLD,SR_JOINED];
                let mut wtr:Vec<u8>=vec![0;4];
                LittleEndian::write_u32(&mut wtr, new_client.guid);
                out.extend_from_slice(&wtr);
                out.extend_from_slice(&new_client.settings[0..]);
                debug!("sending join connection: [{}] to {} @ {}", x, new_client.guid, port);
                &self.broadcast(&out[0..],new_client.addr,COND_SKIPOWNER);
                
                let mut out = vec![MSG_WORLD,SR_JOINED];
                let mut wtr:Vec<u8>=vec![0;4];
                LittleEndian::write_u32(&mut wtr,*port32);
                out.extend_from_slice(&wtr);                        
                out.extend_from_slice(&client.settings[0..]);
                debug!("sending join connection: [{}] to {} @ {}", x, port, new_client.addr);
                &self.broadcast(&out[0..],new_client.addr,COND_OWNERONLY);
            }else{
                    let mut out = vec![MSG_WORLD,SR_REGISTER];
                    let mut wtr:Vec<u8>=vec![0;4];
                    LittleEndian::write_u32(&mut wtr, x);
                    out.extend_from_slice(&wtr);
                    out.extend_from_slice(&new_client.settings[0..]); 
                    let s = String::from_utf8_lossy(&new_client.settings[0..]);
                    println!("result: {} {}", s,&new_client.settings[0..].len());
                    debug!("sending registered connection: [{}]",x);
                    &self.broadcast(&out[0..],addr,COND_OWNERONLY);
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
        let mut uid:u32 =LittleEndian::read_u32(&msg[0 .. 4]);
        let mut oid:u32 =LittleEndian::read_u32(&msg[4 .. 8]);
        if self.connections.contains_key(&oid) {
            let client:Client= self.connections[&oid].clone();
            let s = String::from_utf8_lossy(&client.settings[0..]);
			let mut out = vec![MSG_WORLD,SR_JOINED];
            let mut wtr:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut wtr, client.guid);
            out.extend_from_slice(&wtr);
            out.extend_from_slice(&client.settings[0..]);
            self.broadcast(&out[0..],addr,COND_OWNERONLY);
            drop(self.socket.poll_send_to(&out[0..],&addr));
            debug!("USERSTATE REQUEST Message Event to {} - {:?} ",uid,&out[0..]);
            return true;
        }else{
             let ids:Vec<u32>=self.connections.values().clone().map(|v| v.guid).collect();
             let _ids:Vec<u32>=self.connections.iter().map(|(v,c)| *v).collect();
              debug!("No user state found! requested {} = {} found {} ids ={:?} {:?}",uid,oid, self.connections.len(),&ids[0..],&_ids[0..]);
              return false;
        }
    }
}
impl Objects for Server{
     fn register_property(&mut self,msg:&[u8],addr:SocketAddr)->bool{
        use rand::Rng;
         let start = SystemTime::now();
         let since_the_epoch = start.duration_since(UNIX_EPOCH);
        // let in_ms = since_the_epoch.as_secs() as u128 * 1000 + 
        //     since_the_epoch.subsec_millis() as u128;
        info!("PROPERTY_REGISTRATION Message Event from {}- {} bytes [{:?}]",addr,msg.len(),since_the_epoch);
        let mut sender:u32 =LittleEndian::read_u32(&mut &msg[0..4]);
          let mut sender:u32 =LittleEndian::read_u32(&mut &msg[0..4]);
		let temp = Client{
				instant: Instant::now(),
				guid: sender,
				addr: addr,
				settings: msg.to_vec(),
				last_message:Instant::now(),
			};
		 self.connections.entry(sender)
		 // .or_insert_with(||temp)
		 .and_modify(|e|{  e.addr = addr});
	
        let mut parent_id:u32 =LittleEndian::read_u32(&mut &msg[4..8]);
        let mut pid:u32 =LittleEndian::read_u32(&mut &msg[8..12]);
        let mut payload = &msg[12..];
        let mut rng = rand::thread_rng();
        let mut new_pid = rng.gen::<u32>();
        while self.objects.contains_key(&new_pid){
            new_pid = rng.gen::<u32>();
        }
        let s = String::from_utf8_lossy(&payload[0..]);
        info!("sender:[{}] parent:{} old id {}=>newid {} payload {:?} : {:?}",sender, parent_id,pid,new_pid, s,since_the_epoch);

        let mut client = self.connections.values().cloned().filter(|ref c| c.addr==addr).next().clone().unwrap();
        let new_prop = ObjectRep{
            parent_id: parent_id,
            bytes:payload.to_vec(),
            oid:new_pid,
            owner:sender,
            rid:pid
        };
        //todo add owners to properties.
        self.objects.insert(new_pid,new_prop.clone());
         
        debug!("PROPERTY REGISTRATION [{}] = {}", sender,new_pid);
        let mut out_sender = vec![MSG_WORLD,SR_REGISTER_PROPERTY];
        let mut wsender_id:Vec<u8>=vec![0;4];
        LittleEndian::write_u32(&mut wsender_id, new_prop.owner);
        out_sender.extend_from_slice(&wsender_id);

        let mut wparent_id:Vec<u8>=vec![0;4];
        LittleEndian::write_u32(&mut wparent_id, new_prop.parent_id);
        out_sender.extend_from_slice(&wparent_id);
        //oldid id
        let mut orig_id:Vec<u8>=vec![0;4];
        LittleEndian::write_u32(&mut orig_id, new_prop.rid);
        out_sender.extend_from_slice(&orig_id);
        // new property id
        let mut new_id:Vec<u8>=vec![0;4];
        LittleEndian::write_u32(&mut new_id, new_prop.oid);
        out_sender.extend_from_slice(&new_id);
        // property payload - being name.
        out_sender.extend_from_slice(&payload[0..]);
        debug!("[{:?}] - [{:?}] - [{:?}] [{:?}] {:?}",&wsender_id[0..],&wparent_id[0..], &orig_id[0..],&new_id[0..],&payload[0..]);
        debug!("{:?}",&out_sender[0..]);
        self.broadcast(&out_sender[0..],addr,COND_NONE);

        return true;
     }
     
    fn unregister_object(&mut self,id:u32)->bool{
        if self.objects.contains_key(&id) {
            let object = &self.objects[&id].clone();
            if self.connections.contains_key(&object.owner) {
                let owner=&self.connections[&object.owner].clone();
            }
            let mut out_sender = vec![MSG_WORLD,SR_UNREGISTER_OBJECT];
            let mut wsender_id:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut wsender_id, object.owner);
            out_sender.extend_from_slice(&wsender_id);

            let mut wobj_id:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut wobj_id, object.oid);
            out_sender.extend_from_slice(&wobj_id);
            //self.broadcast(&out_sender[0..],owner.addr,COND_NONE);
            self.objects.remove(&id);

        }
        return true;
    }
     fn register_object(&mut self,msg:&[u8],addr:SocketAddr)->bool{
        use rand::Rng;

        let mut uid:u32 =LittleEndian::read_u32(&msg[0 .. 4]);
        let s = String::from_utf8_lossy(&msg[0..]);
        info!("OBJECT_REGISTRATION Message Event from {}- {} bytes = {:?}",addr,uid,s);
        let mut oid:u32 =LittleEndian::read_u32(&msg[4 .. 8]);
        let mut rng = rand::thread_rng();
        let mut new_oid = rng.gen::<u32>();
        while self.objects.contains_key(&new_oid){
            new_oid = rng.gen::<u32>();
        }
        let mut payload:Vec<u8>=msg[8..].to_vec();
        debug!("[{}] {}=>{} payload {:?}",uid, oid,new_oid, &payload[0..]);
 
		if uid==0{
			let i:Vec<u32>=self.connections.values().cloned()
            .filter(|v| v.addr == addr).map(|v| v.guid).collect();
			uid=i[0];
		}
        let mut new_object = ObjectRep{
            parent_id:uid,
            oid:new_oid,
            bytes:payload.clone(),
            owner: uid,
            rid: oid
        };
        self.objects.insert(new_oid,new_object);
        debug!("1 OBJECT REGISTRATION [{}] {}",uid , new_oid);
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
        let mut requesterid:u32 =LittleEndian::read_u32(&msg[0 .. 4]);
        let mut oid:u32 =LittleEndian::read_u32(&msg[4 .. 8]);
        let mut object:ObjectRep;
        if self.objects.contains_key(&oid) {
            let object: ObjectRep = self.objects[&oid].clone();

			let mut out_sender = vec![MSG_WORLD,SR_REGISTRATION_SPAWNED];

            let mut uid:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut uid, object.parent_id);
            out_sender.extend_from_slice(&uid);

            let mut orig_id:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut orig_id, 0);
            out_sender.extend_from_slice(&orig_id);
            
			let mut new_id:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut new_id, oid);
            out_sender.extend_from_slice(&new_id);
            out_sender.extend_from_slice(&object.bytes[0..]);
            debug!("Sending objectstate {}=> {} [{} {} {}]",object.parent_id, addr,oid,requesterid, object.oid);
         
            debug!("Sending objectstate {:?}",&out_sender[0..]);
            self.broadcast(&out_sender[0..],addr,COND_OWNERONLY);
            return true;
        }else{
            warn!("No object found! {} {}",oid, self.objects.len());
            return false;
        }
     }
       fn propertystate(&mut self,msg:&[u8],addr:SocketAddr)->bool{
        let mut requesterid:u32 =LittleEndian::read_u32(&msg[0 .. 4]);
        let mut oid:u32 =LittleEndian::read_u32(&msg[4 .. 8]);
        let mut object:ObjectRep;

        if self.objects.contains_key(&oid) {
            let new_prop: ObjectRep = self.objects[&oid].clone();

            let mut out_sender = vec![MSG_WORLD,SR_REGISTER_PROPERTY];
            let mut wsender_id:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut wsender_id, new_prop.owner);
            out_sender.extend_from_slice(&wsender_id);

            let mut wparent_id:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut wparent_id, new_prop.parent_id);
            out_sender.extend_from_slice(&wparent_id);
            //oldid id
            let mut orig_id:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut orig_id, new_prop.rid);
            out_sender.extend_from_slice(&orig_id);
            // new property id
            let mut new_id:Vec<u8>=vec![0;4];
            LittleEndian::write_u32(&mut new_id, new_prop.oid);
            out_sender.extend_from_slice(&new_id);
            // property payload - being name.
            out_sender.extend_from_slice(&new_prop.bytes[0..]);

            info!("Sending propertystate {}=> {}",oid, addr);
            self.broadcast(&out_sender[0..],addr,COND_OWNERONLY);
            return true;
        }else{
            warn!("No property found! {} {}",oid, self.objects.len());
            return false;
        }
     }
}


pub enum Event
{
    Future(Box<dyn future::Future<Item = Event, Error = Event>>),
    Stream(Box<dyn stream::Stream<Item = Event, Error = Event>>),
}

impl Stream for Server {
    type Item = ();//std::io::Error;//();//Event;
    type Error = ();

       fn poll(&mut self) ->Poll<Option<Self::Item>, Self::Error>{
       // std::result::Result<futures::Async<std::option::Option<()>>, Event>{

       //Poll<(), Self::Error> {
       //std::result::Result<futures::Async<std::option::Option<Event>>, Event> {
        loop{
        let result = match self.socket.poll_recv_from(&mut self.buf){
            Ok(Async::Ready(ready_result)) =>{
                println!("poll_recv_from ready: {:?}",ready_result);
                ready_result
            }
            Ok(Async::NotReady)=>{
                println!("poll_recv_from Notready");
                return Ok(Async::NotReady);
            }
            Err(e)=>{
                panic!("io error: {:?}",e);
            }
        };
        let (nbytes, addr) = result;
        self.close_stale_connections();
        self.update_client(addr);
        println!("received {} bytes from {}",nbytes,addr);
        let mut cop = self.buf[..nbytes].to_vec();
        let sr=self.parse_route(cop.clone(),addr);
        }
       }

        //  loop {
        //     if let Some((size, peer)) = self.to_send {
		// 		self.update_client(peer);
        //       //  self.close_stale_connections();
		// 			if size>1 {
		// 				let mut cop = self.buf[..size].to_vec();
		// 			     debug!("Recd [{}][{}]{} bytes FROM {}",&cop[0],&cop[1], size, peer);
		// 				let sr=self.parse_route(cop.clone(),peer);
		// 				if !sr {
		// 					debug!("Echoed [{}][{}]{} bytes BACK to {}",&cop[0],&cop[1], size, peer);
		// 					self.broadcast(&cop[0..],peer,COND_NONE);
		// 				}
		// 			}else{
        //                 println!("PONG {:?}",&peer);
		// 			}
		// 			self.to_send = None;
		// 			debug!("///{}",self);
            
        //     self.to_send = Some(self.socket.poll_recv_from(&mut self.buf));
        //      Ok(())

        //     }
        //  }

    // }
}
pub struct Listener{
    stream: Server,
//    socket: UdpSocket
}
impl Listener{
    pub fn new(stream: Server) ->Listener{
        Listener{
            stream,
            // socket:stream.socket
        }
    }
}
impl Future for Listener 
    {
        type Item=();
        type Error =();
        fn poll(&mut self)->Poll<(),Self::Error>{
            loop{
                let value = match try_ready!(self.stream.poll()){
                    Some(value)=> value,
                    None=>break
                };
                println!("Server={}",self.stream);
            }
            Ok(Async::Ready(()))
        }
    }