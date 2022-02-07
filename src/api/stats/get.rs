use crate::{
    api::{
        http_reply,
        ws::{command::STATS_GET, Websocket},
        ws_reply_with, Reply, lock_or_exit,
        http_route_hit_log
    },
    AppData,
    lib::Stats
};
use actix_web::{web::Data, HttpResponse};
use actix_web_actors::ws::WebsocketContext;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupDefaults {
    priority: u32,
    max_byte_size: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupData {
    priority: u32,
    byte_size: u32,
    max_byte_size: Option<u32>,
    msg_count: usize,
}

pub fn handle(data: Data<AppData>) -> Reply<Stats> {
    let stats = {
        let stats = lock_or_exit(&data.stats);
        stats.clone()
    };
    Reply::OkWData(stats)
}

const ROUTE: &'static str = "GET /api/stats";
pub fn http_handle(data: Data<AppData>) -> HttpResponse {
    http_route_hit_log::<()>(ROUTE, None);
    http_reply(ROUTE, handle(data))
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>) {
    http_route_hit_log::<()>(STATS_GET, None);
    ws_reply_with(ctx, STATS_GET)(handle(data));
}
