use crate::{
    api::{
        get_require_msg, get_required_priority, http_reply,
        ws::{command::MSG_POST, Websocket},
        ws_reply_with, Reply,
    },
    AppData,
};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use actix_web_actors::ws;
use msg_store::{errors::Error, Packet};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::exit;

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    priority: u32,
    msg: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReturnBody {
    uuid: String,
}

pub fn handle(data: Data<AppData>, body: Value) -> Reply<ReturnBody> {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => exit(1),
    };
    let priority = match get_required_priority(&body) {
        Ok(priority) => priority,
        Err(message) => return Reply::BadRequest(message),
    };
    let msg = match get_require_msg(&body) {
        Ok(msg) => msg,
        Err(message) => return Reply::BadRequest(message),
    };
    let body = Body { priority, msg };
    let uuid = match store.add(Packet::new(body.priority, body.msg.to_string())) {
        Ok(uuid) => uuid,
        Err(error) => match error {
            Error::ExceedesStoreMax => {
                return Reply::Conflict(
                    "Message byte size exceeds the max byte size limit allowed by the store"
                        .to_string(),
                );
            }
            Error::ExceedesGroupMax => {
                return Reply::Conflict(
                    "Message byte size exceeds the max byte size limit allowed by the group"
                        .to_string(),
                );
            }
            Error::LacksPriority => {
                return Reply::Conflict(
                    "The store has reached max capcity and could not accept message".to_string(),
                );
            }
            _ => exit(1),
        },
    };
    Reply::OkWData(ReturnBody {
        uuid: uuid.to_string(),
    })
}

pub fn http_handle(data: Data<AppData>, body: Json<Value>) -> HttpResponse {
    http_reply(handle(data, body.into_inner()))
}

pub fn ws_handle(ctx: &mut ws::WebsocketContext<Websocket>, data: Data<AppData>, obj: Value) {
    ws_reply_with(ctx, MSG_POST)(handle(data, obj));
}
