use actix_web::{
    HttpResponse,
    web::{
        Data,
        Json
    }
};
use crate::{
    AppData,
    StoreGaurd,
    fmt_result
};
use msg_store::{Packet, Uuid};
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

pub fn post_msg(store: &mut StoreGaurd, priority: &i32, msg: &str) -> Result<Uuid, String> {
    let packet = Packet::new(*priority, msg.to_string());
    fmt_result!(store.add(&packet))
}

pub fn post(data: Data<AppData>, body: Json<Body>) -> HttpResponse {
    let mut store = match fmt_result!(data.store.try_lock()) {
        Ok(db) => db,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let uuid = match fmt_result!(post_msg(&mut store, &body.priority, &body.msg)) {
        Ok(id) => id,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(Reply::Ok { uuid: uuid.to_string() })
}