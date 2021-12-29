use crate::{
    api::{
        get_optional_priority, http_reply,
        ws::{command::GROUP_DEFAULTS_GET, Websocket},
        ws_reply_with, Reply,
    },
    AppData,
};
use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use actix_web_actors::ws::WebsocketContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::exit;

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    priority: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct GroupDefaults {
    priority: u32,
    max_byte_size: Option<u32>,
}

pub fn handle(data: Data<AppData>, info: Info) -> Reply<Vec<GroupDefaults>> {
    let store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            exit(1);
        }
    };
    if let Some(priority) = info.priority {
        if let Some(defaults) = store.group_defaults.get(&priority) {
            let group_defaults = GroupDefaults {
                priority: priority.clone(),
                max_byte_size: defaults.max_byte_size,
            };
            Reply::OkWData(vec![group_defaults])
        } else {
            Reply::OkWData(vec![])
        }
    } else {
        let data = store
            .group_defaults
            .iter()
            .map(|(priority, defaults)| GroupDefaults {
                priority: priority.clone(),
                max_byte_size: defaults.max_byte_size,
            })
            .collect::<Vec<GroupDefaults>>();
        Reply::OkWData(data)
    }
}

pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    http_reply(handle(data, info.into_inner()))
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, info: Value) {
    let mut reply = ws_reply_with(ctx, GROUP_DEFAULTS_GET);
    let priority = match get_optional_priority(&info) {
        Ok(priority) => priority,
        Err(message) => return reply(Reply::BadRequest(message)),
    };
    reply(handle(data, Info { priority }))
}
