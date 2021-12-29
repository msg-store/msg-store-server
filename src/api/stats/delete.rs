use crate::{
    api::{
        http_reply,
        stats::get::Stats,
        ws::{command::STATS_DELETE, Websocket},
        ws_reply_with, Reply,
    },
    AppData,
};
use actix_web::{web::Data, HttpResponse};
use actix_web_actors::ws;
use std::process::exit;

pub fn handle(data: Data<AppData>) -> Reply<Stats> {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            exit(1);
        }
    };
    let data = Stats {
        inserted: store.msgs_inserted,
        deleted: store.msgs_deleted,
        pruned: store.msgs_pruned,
    };
    store.msgs_inserted = 0;
    store.msgs_deleted = 0;
    store.msgs_pruned = 0;
    Reply::OkWData(data)
}

pub fn http_handle(data: Data<AppData>) -> HttpResponse {
    http_reply(handle(data))
}

pub fn ws_handle(ctx: &mut ws::WebsocketContext<Websocket>, data: Data<AppData>) {
    ws_reply_with(ctx, STATS_DELETE)(handle(data));
}
