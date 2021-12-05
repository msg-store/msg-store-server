use actix_web::{
    HttpResponse,
    web::{
        Data,
        Query
    }
};
use crate::AppData;
use msg_store::{
    Uuid
};
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct MsgData {
    uuid: String,
    msg: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    uuid: Option<String>,
    priority: Option<i32>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { data: Option<MsgData> }
}

pub fn get(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let uuid = match &info.uuid {
        Some(str) => Some(Uuid::from_string(&str)),
        None => None
    };
    let stored_packet = match store.get(uuid, info.priority) {
        Ok(data) => data,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let data = if let Some(stored_packet) = stored_packet {
        Some(MsgData {
            uuid: stored_packet.uuid.to_string(),
            msg: stored_packet.msg
        })
    } else {
        None
    };
    HttpResponse::Ok().json(Reply::Ok{ data })
}