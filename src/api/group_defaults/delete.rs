use crate::{
    api::{
        get_required_priority, http_reply, update_config,
        ws::{command::GROUP_DEFAULTS_DELETE, Websocket},
        ws_reply_with, Reply,
    },
    config::GroupConfig,
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
    priority: u32,
}

pub fn handle(data: Data<AppData>, info: Info) -> Reply<()> {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            exit(1);
        }
    };
    store.delete_group_defaults(info.priority);
    let mut config = match data.config.try_lock() {
        Ok(config) => config,
        Err(_error) => {
            exit(1);
        }
    };
    let groups = match &config.groups {
        Some(groups) => groups,
        None => {
            return Reply::Ok;
        }
    };
    let new_groups: Vec<GroupConfig> = groups
        .iter()
        .filter(|group| {
            if group.priority != info.priority {
                true
            } else {
                false
            }
        })
        .map(|group| group.clone())
        .collect();
    config.groups = Some(new_groups);
    if let Err(_error) = update_config(&config, &data.config_location) {
        exit(1);
    }
    Reply::Ok
}

pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    http_reply(handle(data, info.into_inner()))
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, info: Value) {
    let mut reply = ws_reply_with(ctx, GROUP_DEFAULTS_DELETE);
    let priority = match get_required_priority(&info) {
        Ok(priority) => priority,
        Err(message) => return reply(Reply::BadRequest(message)),
    };
    reply(handle(data, Info { priority }));
}
