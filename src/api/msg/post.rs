use actix_web::{
    HttpResponse,
    web::{
        Data,
        Json
    }
};
use crate::AppData;
use msg_store::Packet;
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    priority: i32,
    msg: String
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { uuid: String }
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
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(Reply::Ok { uuid: uuid.to_string() })
}