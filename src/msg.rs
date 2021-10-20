use msg_store::store::Priority;
use serde::{Deserialize,Serialize};
use serde_json::{
    value::Value,
    to_string,
    from_value,
    from_str
};

    
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CMD {
    Post,
    Get,
    Delete
}

pub mod request {
    use super::*;

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Packet {
        cmd: CMD,
        data: Option<Value>
    }

    impl Packet {

        pub fn from_str(str: &str) -> Result<Packet, String> {
            match from_str(str) {
                Ok(packet) => Ok(packet),
                Err(error) => Err(error.to_string())
            }
        }

    }

    pub fn from_value<T>(data: T) -> Result<T, String>
    {
        match from_value(data) {
            Ok(data) => Ok(data),
            Err(error) => Err(error.to_string())
        }
    } 

    #[derive(Deserialize, Serialize)]
    pub struct Post {
        cmd: CMD,
        priority: Priority,
        msg: String
    } 

    #[derive(Deserialize, Serialize)]
    pub struct Get {
        cmd: CMD
    }

    #[derive(Deserialize, Serialize)]
    pub struct GetFrom {
        cmd: CMD,
        priority: Priority
    }

    #[derive(Deserialize, Serialize)]
    pub struct Delete {
        cmd: CMD,
        uuid: String
    }

    #[derive(Deserialize, Serialize)]
    pub struct Put {
        cmd: CMD,
        uuid: String,
        new_priority: Priority
    }

}

pub mod response {

    use super::*;

    #[derive(Deserialize, Serialize)]
    pub struct Get {
        cmd: CMD,
	uuid: String
    }	    

    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Post {
        cmd: CMD,
	uuid: String
    }

    #[derive(Deserialize, Serialize)]
    pub struct Put {
        cmd: CMD,
        uuid: String
    }
	
}

