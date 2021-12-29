use crate::{
    api::{
        get_required_uuid, http_bad_request, http_reply, validate_uuid_string,
        ws::{command::MSG_DELETE, Websocket},
        ws_reply_with, Reply,
    },
    AppData,
};
use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use actix_web_actors::ws;
use msg_store::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::exit;

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    uuid: String,
}

pub fn handle(data: Data<AppData>, uuid: Uuid) -> Reply<()> {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => exit(1),
    };
    if let Err(_error) = store.del(&uuid) {
        exit(1);
    }
    Reply::Ok
}

pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let uuid = match validate_uuid_string(info.into_inner().uuid) {
        Ok(uuid) => uuid,
        Err(message) => {
            return http_bad_request(message);
        }
    };
    http_reply(handle(data, uuid))
}

pub fn ws_handle(ctx: &mut ws::WebsocketContext<Websocket>, data: Data<AppData>, info: Value) {
    let mut reply = ws_reply_with(ctx, MSG_DELETE);
    let uuid = match get_required_uuid(&info) {
        Ok(uuid) => uuid,
        Err(message) => {
            return reply(Reply::BadRequest(format!("/data/{}", message)));
        }
    };
    reply(handle(data, uuid));
}
