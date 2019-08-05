

use std::time::{Instant, Duration};
use std::net::SocketAddr;
use std::fmt::{Formatter,Error,Display};

#[derive(Hash, Eq, PartialEq, Debug,Clone)]
pub struct Client {
    pub instant: Instant,
    pub guid: u32,
    pub addr: SocketAddr,
    pub settings:Vec<u8>,
    pub last_message:Instant
}

impl Client{
    pub fn new(_guid:u32,_addr:SocketAddr,_settings:Vec<u8>)->Client{
        let client:Client = Client{
            instant: Instant::now(),
            guid:_guid,
            addr: _addr,
            settings:_settings,
            last_message:Instant::now()
        };
        println!("Client Created: {}",client);
        return client;
    }
    pub fn last_update(&mut self){
        self.last_message = Instant::now()
    }
	pub fn update(&mut self)->bool{
		let now = Instant::now();
		
		let b = now.duration_since(self.last_message).as_secs() > 0;
		self.last_message = now;
        println!("Updating {:?}",&self.addr.to_string());
		return b;
	}
	pub fn is_same(&mut self,addr: SocketAddr)->bool{
		return self.addr == addr;
		}
}


impl Display for Client {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "[{}] {} ", self.guid, self.addr)
    }
}
impl Drop for Client {
    fn drop(&mut self){
        println!("Dropping client {}",self);
    }
}

#[derive(Eq, PartialEq, Debug,Clone)]
pub struct ObjectRep{
    pub oid:u32, // object id
    pub parent_id:u32, //owning object, if any
    pub bytes:Vec<u8>, //data
    pub owner:u32, //registering player
    pub rid:u32  //Origination request id.
}
impl Display for ObjectRep{
	  fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "ObjectID: {} Parent: {} Owner {} RID {} Size {}", self.oid, self.parent_id,self.owner,self.rid,self.bytes.len())
    }
}

impl Drop for ObjectRep{
    fn drop(&mut self){
        println!("Dropping Object {}",self);
    }
}
