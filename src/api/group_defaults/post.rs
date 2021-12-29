use crate::{
    api::{
        get_optional_max_byte_size, get_required_priority, http_reply, update_config,
        ws::{command::GROUP_DEFAULTS_POST, Websocket},
        ws_reply_with, Reply,
    },
    config::GroupConfig,
    AppData,
};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use actix_web_actors::ws::WebsocketContext;
use msg_store::store::GroupDefaults;
use serde_json::Value;
use std::{borrow::BorrowMut, process::exit};

pub struct Body {
    priority: u32,
    max_byte_size: Option<u32>,
}

pub fn validate_body(body: Value) -> Result<Body, String> {
    let priority = get_required_priority(&body)?;
    let max_byte_size = get_optional_max_byte_size(&body)?;
    Ok(Body {
        priority,
        max_byte_size,
    })
}

pub fn handle(data: Data<AppData>, body: Value) -> Reply<()> {
    let body = match validate_body(body) {
        Ok(body) => body,
        Err(message) => return Reply::BadRequest(message),
    };
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            exit(1);
        }
    };
    let mut config = match data.config.try_lock() {
        Ok(config) => config,
        Err(_error) => {
            exit(1);
        }
    };
    let defaults = GroupDefaults {
        max_byte_size: body.max_byte_size,
    };
    if let Err(_error) = store.update_group_defaults(body.priority, &defaults) {
        exit(1);
    }
    let mk_group_config = || -> GroupConfig {
        GroupConfig {
            priority: body.priority,
            max_byte_size: body.max_byte_size,
        }
    };
    if let Some(groups) = config.groups.borrow_mut() {
        let mut group_index: Option<usize> = None;
        for i in 0..groups.len() {
            let group = match groups.get(i) {
                Some(group) => group,
                None => {
                    continue;
                }
            };
            if body.priority == group.priority {
                group_index = Some(i);
                break;
            }
        }
        if let Some(index) = group_index {
            if let Some(group) = groups.get_mut(index) {
                group.max_byte_size = body.max_byte_size;
            } else {
                groups.push(mk_group_config());
            }
        } else {
            groups.push(mk_group_config());
        }
        groups.sort();
    } else {
        config.groups = Some(vec![mk_group_config()]);
    }
    if let Err(_error) = update_config(&config, &data.config_location) {
        exit(1);
    }
    Reply::Ok
}

pub fn http_handle(data: Data<AppData>, body: Json<Value>) -> HttpResponse {
    http_reply(handle(data, body.into_inner()))
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, body: Value) {
    (ws_reply_with(ctx, GROUP_DEFAULTS_POST)(handle(data, body)));
}
