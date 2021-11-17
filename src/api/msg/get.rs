use actix_web::{
    HttpResponse,
    web::{
        Data,
        Query
    }
};
use crate::{
    AppData,
    StoreGaurd,
    fmt_result
};
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

pub fn get_msg(store: &mut StoreGaurd, info: &Info) -> Option<MsgData> {
    let uuid = match &info.uuid {
        Some(str) => Some(Uuid::from_string(&str)),
        None => None
    };
    match store.get(uuid, info.priority) {
        Some(stored_packet) => {
            Some(MsgData {
                uuid: stored_packet.uuid.to_string(),
                msg: stored_packet.msg
            })
        },
        None => None
    }
}

pub fn get(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let mut store = match fmt_result!(data.store.try_lock()) {
        Ok(db) => db,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let data = get_msg(&mut store, &info.into_inner());
    HttpResponse::Ok().json(Reply::Ok{ data })
}