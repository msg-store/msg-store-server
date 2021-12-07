use actix_web::{
    HttpResponse,
    web::{
        Data,
        Query
    }
};
use crate::AppData;
use msg_store::{
    GetOptions,
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
    priority: Option<u32>,
    reverse: Option<bool>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok(Option<MsgData>)
}

pub fn get(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut options = GetOptions::default();
    if let Some(uuid_string) = info.uuid.clone() {
        let uuid = match Uuid::from_string(&uuid_string) {
            Ok(uuid) => uuid,
            Err(_error) => {
                return HttpResponse::BadRequest().content_type("text/plain").body("Query deserialize error: Invalid UUID");
            }
        };
        options.uuid = Some(uuid);
    }
    if let Some(priority) = info.priority {
        options.priority = Some(priority);
    }
    if let Some(reverse) = info.reverse {
        if reverse {
            options.reverse = true;
        }
    }
    let stored_packet = match store.get(options) {
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
    HttpResponse::Ok().json(Reply::Ok(data))
}