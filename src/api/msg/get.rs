use crate::{
    api::{
        from_value_prop, get_optional_priority, get_optional_uuid, http_reply,
        validate_uuid_string,
        ws::{command::MSG_GET, Websocket},
        ws_reply_with, Reply,
    },
    AppData,
};
use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use actix_web_actors::ws::WebsocketContext;
use msg_store::{GetOptions, Uuid};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::exit;

#[derive(Debug, Deserialize, Serialize)]
pub struct MsgData {
    uuid: String,
    msg: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    uuid: Option<String>,
    priority: Option<u32>,
    reverse: Option<bool>,
}

pub struct InnerInfo {
    uuid: Option<Uuid>,
    priority: Option<u32>,
    reverse: Option<bool>,
}

pub fn make_inner_info(info: Info) -> Result<InnerInfo, String> {
    let uuid = if let Some(uuid_string) = info.uuid {
        Some(validate_uuid_string(uuid_string)?)
    } else {
        None
    };
    let priority = info.priority;
    let reverse = info.reverse;
    Ok(InnerInfo {
        uuid,
        priority,
        reverse,
    })
}

pub fn handle(data: Data<AppData>, info: InnerInfo) -> Reply<Option<MsgData>> {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            exit(1);
        }
    };
    let mut options = GetOptions::default();
    if let Some(uuid) = info.uuid.clone() {
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
            exit(1);
        }
    };
    let data = if let Some(stored_packet) = stored_packet {
        Some(MsgData {
            uuid: stored_packet.uuid.to_string(),
            msg: stored_packet.msg,
        })
    } else {
        None
    };
    Reply::OkWData(data)
}

pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let info = match make_inner_info(info.into_inner()) {
        Ok(info) => info,
        Err(message) => {
            return http_reply::<()>(Reply::BadRequest(message));
        }
    };
    http_reply(handle(data, info))
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>,data: Data<AppData>, info: Value) {
    let mut reply = ws_reply_with(ctx, MSG_GET);
    let priority = match get_optional_priority(&info) {
        Ok(priority) => priority,
        Err(message) => {
            return reply(Reply::BadRequest(message));
        }
    };
    let uuid = match get_optional_uuid(&info) {
        Ok(uuid) => uuid,
        Err(message) => return reply(Reply::BadRequest(message)),
    };
    let reverse = match from_value_prop::<bool, _>(&info, "reverse", "boolean") {
        Ok(reverse) => reverse,
        Err(message) => return reply(Reply::BadRequest(message)),
    };
    reply(handle(
        data,
        InnerInfo {
            priority,
            uuid,
            reverse,
        },
    ));
}
