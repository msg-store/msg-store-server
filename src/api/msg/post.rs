use actix_web::{
    HttpResponse,
    web::{
        Data,
        Json
    }
};
use crate::AppData;
use msg_store::{Packet, errors::Error};
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    priority: u32,
    msg: String
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { uuid: String },
    Err { code: u32, message: String }
}
impl Reply {
    pub fn exceeds_store_max() -> Reply {
        Reply::Err{ code: 1, message: "Message byte size exceeds the max byte size limit allowed by the store".to_string() }
    }
    pub fn exceeds_group_max() -> Reply {
        Reply::Err{ code: 2, message: "Message byte size exceeds the max byte size limit allowed by the group".to_string() }
    }
    pub fn lacks_priority() -> Reply {
        Reply::Err{ code: 3, message: "The store has reached max capcity and could not accept message".to_string() }
    }
}

pub fn post(data: Data<AppData>, body: Json<Body>) -> HttpResponse {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let uuid = match store.add(Packet::new(body.priority, body.msg.to_string())) {
        Ok(uuid) => uuid,
        Err(error) => {
            match error {
                Error::ExceedesStoreMax => { return HttpResponse::Conflict().json(Reply::exceeds_store_max()); },
                Error::ExceedesGroupMax => { return HttpResponse::Conflict().json(Reply::exceeds_group_max()); },
                Error::LacksPriority => { return HttpResponse::Conflict().json(Reply::lacks_priority()); },
                _ => { return HttpResponse::InternalServerError().finish(); }
            }            
        }
    };
    HttpResponse::Ok().json(Reply::Ok { uuid: uuid.to_string() })
}