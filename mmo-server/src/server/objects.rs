

use std::time::{Instant, Duration};
use std::net::SocketAddr;
use std::fmt::{Formatter,Error,Display};
use byteorder::{ByteOrder, LittleEndian,WriteBytesExt};
use serde::{Serialize, Deserialize};

#[derive(Hash, Eq, PartialEq, Debug,Clone,Serialize, Deserialize)]
pub struct KeyStore{
    pub keys:Vec<ObjectKey>
}
impl KeyStore{
    pub fn new()->KeyStore{
        KeyStore{
            keys: vec![]
        }
    }
    pub fn get_client_ids(self)->Vec<String>{
        self.keys.iter().filter(|s| s.objtype == String::from("client")).map(|s| s.clone().id).collect()
    }
    pub fn get_property_ids(self)->Vec<String>{
        self.keys.iter().filter(|s| s.objtype == String::from("property")).map(|s| s.clone().id).collect()

    }
    pub fn get_object_ids(self)->Vec<String>{
        self.keys.iter().filter(|s| s.objtype == String::from("object")).map(|s| s.clone().id).collect()

    }
    
}
#[derive(Hash, Eq, PartialEq, Debug,Clone,Serialize, Deserialize)]
pub struct ObjectKey{
    pub id: String,
    pub objtype: String

}
#[derive(Hash, Eq, PartialEq, Debug,Clone,Serialize, Deserialize)]
pub struct Client {
    #[serde(skip,default="Instant::now")]
    pub instant: Instant,
    pub guid: u32,
    pub addr: SocketAddr,
    pub settings:Vec<u8>,
    #[serde(skip,default="Instant::now")]
    pub last_message:Instant,
}

impl Client{
    pub fn new(_guid:u32,_addr:SocketAddr,_settings:Vec<u8>)->Client{
        
        let mut _serialized:Vec<u8>=vec![0;4];
        LittleEndian::write_u32(&mut _serialized,_guid);
        _serialized.extend_from_slice(&_settings[0..]);

        let client:Client = Client{
            instant: Instant::now(),
            guid:_guid,
            addr: _addr,
            settings:_settings,
            last_message:Instant::now(),
        };
        debug!("Client Created: {}",client);
        return client;
    }
    pub fn serialized(self)->Result<String,serde_json::error::Error>{
        serde_json::to_string(&self)
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
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        write!(fmt, "[{}] {} ", self.guid, self.addr)
    }
}
impl Drop for Client {
    fn drop(&mut self){
        println!("Dropping client {}",self);
    }
}
#[derive(Eq, PartialEq, Debug,Clone,Serialize, Deserialize)]
pub struct ObjectRep{
    pub id:u32, // object id
    pub parent_id:u32, //owning object, if any
    pub bytes:Vec<u8>, //data
    pub client_id:u32, //registering player
    pub original_id:u32  //Origination request id
}
impl Display for ObjectRep{
	  fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), Error> {
        write!(fmt, "ObjectID: {} Parent: {} Owner {} RID {} Size {}", self.id, self.parent_id,self.client_id,self.original_id,self.bytes.len())
    }
}
impl Drop for ObjectRep{
    fn drop(&mut self){
        println!("Dropping Object {}",self);
    }
}
impl ObjectRep{
    pub fn new(object_id: u32, new_id: u32,parent: u32, _bytes:Vec<u8>,_client_id: u32)->ObjectRep{
        ObjectRep{
             id: new_id,
             parent_id:parent,
             bytes: _bytes,
             client_id:_client_id,
             original_id:object_id
        }
    }
    pub fn serialized(self)->Result<String,serde_json::error::Error>{
        serde_json::to_string(&self)
    }
}
