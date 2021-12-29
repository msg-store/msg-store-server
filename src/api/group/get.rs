use crate::{
    api::{
        from_value_prop, http_reply,
        ws::{command::GROUP_GET, Websocket},
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
#[serde(rename_all = "camelCase")]
pub struct Info {
    priority: Option<u32>,
    include_msg_data: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Msg {
    uuid: String,
    byte_size: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    priority: u32,
    byte_size: u32,
    max_byte_size: Option<u32>,
    msg_count: u32,
    messages: Vec<Msg>,
}

pub fn handle(data: Data<AppData>, info: Info) -> Reply<Vec<Group>> {
    let store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            exit(1);
        }
    };
    if let Some(priority) = info.priority {
        if let Some(group) = store.groups_map.get(&priority) {
            let group = Group {
                priority: priority.clone(),
                byte_size: group.byte_size,
                max_byte_size: group.max_byte_size,
                msg_count: group.msgs_map.len() as u32,
                messages: match info.include_msg_data {
                    Some(include_msg_data) => match include_msg_data {
                        true => group
                            .msgs_map
                            .iter()
                            .map(|(uuid, byte_size)| Msg {
                                uuid: uuid.to_string(),
                                byte_size: byte_size.clone(),
                            })
                            .collect::<Vec<Msg>>(),
                        false => vec![],
                    },
                    None => vec![],
                },
            };
            Reply::OkWData(vec![group])
        } else {
            Reply::OkWData(vec![])
        }
    } else {
        let data = store
            .groups_map
            .iter()
            .map(|(priority, group)| Group {
                priority: priority.clone(),
                byte_size: group.byte_size,
                max_byte_size: group.max_byte_size,
                msg_count: group.msgs_map.len() as u32,
                messages: match info.include_msg_data {
                    Some(include_msg_data) => match include_msg_data {
                        true => group
                            .msgs_map
                            .iter()
                            .map(|(uuid, byte_size)| Msg {
                                uuid: uuid.to_string(),
                                byte_size: byte_size.clone(),
                            })
                            .collect::<Vec<Msg>>(),
                        false => vec![],
                    },
                    None => vec![],
                },
            })
            .collect::<Vec<Group>>();
        Reply::OkWData(data)
    }
}

pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    http_reply(handle(data, info.into_inner()))
}

pub fn validate_params(value: Value) -> Result<Info, String> {
    let priority = from_value_prop::<u32, _>(&value, "priority", "number")?;
    let mut include_msg_data = from_value_prop::<bool, _>(&value, "include_msg_data", "boolean")?;
    if let None = include_msg_data {
        include_msg_data = from_value_prop::<bool, _>(&value, "includeMsgData", "boolean")?;
    }
    Ok(Info {
        priority,
        include_msg_data,
    })
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, info: Value) {
    let mut reply = ws_reply_with(ctx, GROUP_GET);
    let info = match validate_params(info) {
        Ok(info) => info,
        Err(message) => {
            reply(Reply::BadRequest(message));
            return;
        }
    };
    reply(handle(data, info));
}
