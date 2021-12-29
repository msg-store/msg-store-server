use crate::{
    api::{
        http_reply,
        ws::{command::STATS_GET, Websocket},
        ws_reply_with, Reply,
    },
    AppData,
};
use actix_web::{web::Data, HttpResponse};
use actix_web_actors::ws::WebsocketContext;

use serde::{Deserialize, Serialize};
use std::process::exit;

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupDefaults {
    priority: u32,
    max_byte_size: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupData {
    priority: u32,
    byte_size: u32,
    max_byte_size: Option<u32>,
    msg_count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Stats {
    pub inserted: u32,
    pub deleted: u32,
    pub pruned: u32,
}

pub fn handle(data: Data<AppData>) -> Reply<Stats> {
    let store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            exit(1);
        }
    };
    Reply::OkWData(Stats {
        inserted: store.msgs_inserted,
        deleted: store.msgs_deleted,
        pruned: store.msgs_pruned,
    })
}

pub fn http_handle(data: Data<AppData>) -> HttpResponse {
    http_reply(handle(data))
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>) {
    ws_reply_with(ctx, STATS_GET)(handle(data));
}
