use crate::{
    api::{
        http_reply,
        Stats,
        ws::{command::STATS_DELETE, Websocket},
        ws_reply_with, Reply, lock_or_exit, http_route_hit_log,
    },
    AppData
};
use actix_web::{web::Data, HttpResponse};
use actix_web_actors::ws;

pub fn handle(data: Data<AppData>) -> Reply<Stats> {
    let stats = {
        let mut stats = lock_or_exit(&data.stats);
        let old_stats = stats.clone();
        stats.inserted = 0;
        stats.deleted = 0;
        stats.pruned = 0;
        old_stats
    };
    Reply::OkWData(stats)
}

const ROUTE: &'static str = "DEL /api/stats";
pub fn http_handle(data: Data<AppData>) -> HttpResponse {
    http_route_hit_log::<()>(ROUTE, None);
    http_reply(ROUTE, handle(data))
}

pub fn ws_handle(ctx: &mut ws::WebsocketContext<Websocket>, data: Data<AppData>) {
    http_route_hit_log::<()>(STATS_DELETE, None);
    ws_reply_with(ctx, STATS_DELETE)(handle(data));
}
