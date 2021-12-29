use crate::{
    api::{
        get_optional_max_byte_size, http_reply, update_config,
        ws::{command::STORE_PUT, Websocket},
        ws_reply_with, Reply,
    },
    AppData,
};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use actix_web_actors::ws::WebsocketContext;
use msg_store::store::StoreDefaults;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::exit;

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    max_byte_size: Option<u32>,
}

pub fn validate_body(value: Value) -> Result<Body, Reply<()>> {
    let max_byte_size = match get_optional_max_byte_size(&value) {
        Ok(max_byte_size) => Ok(max_byte_size),
        Err(message) => Err(Reply::BadRequest(format!("/data/{}", message))),
    }?;
    Ok(Body { max_byte_size })
}

pub fn handle(data: Data<AppData>, body: Body) -> Reply<()> {
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
    let max_byte_size = body.max_byte_size;
    store.max_byte_size = max_byte_size;
    let defaults = StoreDefaults {
        max_byte_size: max_byte_size,
    };
    if let Err(_error) = store.update_store_defaults(&defaults) {
        exit(1);
    }
    config.max_byte_size = max_byte_size;
    if let Err(_error) = update_config(&mut config, &data.config_location) {
        exit(1);
    }
    Reply::Ok
}

pub fn http_handle(data: Data<AppData>, body: Json<Value>) -> HttpResponse {
    let reply = match validate_body(body.into_inner()) {
        Ok(body) => handle(data, body),
        Err(reply) => reply,
    };
    http_reply(reply)
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, body: Value) {
    let reply = match validate_body(body) {
        Ok(body) => handle(data, body),
        Err(reply) => reply,
    };
    ws_reply_with(ctx, STORE_PUT)(reply);
}
