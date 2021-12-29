use crate::{
    api::{
        from_value_prop_required, http_reply,
        ws::{command::GROUP_DELETE, Websocket},
        ws_reply_with, Reply,
    },
    AppData,
};
use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use actix_web_actors::ws::WebsocketContext;
use msg_store::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::exit;

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    priority: u32,
}

pub fn handle(data: Data<AppData>, info: Info) -> Reply<()> {
    let list = {
        let store = match data.store.try_lock() {
            Ok(store) => store,
            Err(_error) => {
                exit(1);
            }
        };
        // get list of messages to remove
        let list = if let Some(group) = store.groups_map.get(&info.priority) {
            group
                .msgs_map
                .keys()
                .map(|uuid| -> Uuid { uuid.clone() })
                .collect::<Vec<Uuid>>()
        } else {
            return Reply::Ok;
        };
        list
    };
    for uuid in list.iter() {
        let mut store = match data.store.try_lock() {
            Ok(store) => store,
            Err(_error) => {
                exit(1);
            }
        };
        if let Err(_error) = store.del(uuid) {
            exit(1);
        }
    }
    Reply::Ok
}

pub fn handle_http(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    http_reply(handle(data, info.into_inner()))
}

pub fn handle_ws(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, info: Value) {
    let mut reply = ws_reply_with(ctx, GROUP_DELETE);
    let priority = match from_value_prop_required(&info, "priority", "number") {
        Ok(priority) => priority,
        Err(message) => {
            reply(Reply::BadRequest(message));
            return;
        }
    };
    reply(handle(data, Info { priority }));
}
