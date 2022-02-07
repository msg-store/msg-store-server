use crate::{
    api::{
        http_reply,
        ws::{command::STORE_GET, Websocket},
        ws_reply_with, Reply, lock_or_exit, http_route_hit_log,
    },
    AppData,
};
use actix_web::{web::Data, HttpResponse};
use actix_web_actors::ws::WebsocketContext;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupDefaults {
    priority: u32,
    max_byte_size: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupData {
    priority: u32,
    byte_size: u32,
    max_byte_size: Option<u32>,
    msg_count: usize,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StoreData {
    byte_size: u32,
    max_byte_size: Option<u32>,
    msg_count: usize,
    group_count: usize,
    groups: Vec<GroupData>,
    group_defaults: Vec<GroupDefaults>,
}

pub fn handle(data: Data<AppData>) -> Reply<StoreData> {
    let store = lock_or_exit(&data.store);
    let groups = store
        .groups_map
        .iter()
        .map(|(priority, group)| GroupData {
            priority: *priority,
            byte_size: group.byte_size,
            max_byte_size: group.max_byte_size,
            msg_count: group.msgs_map.len(),
        })
        .collect::<Vec<GroupData>>();
    let group_defaults = store
        .group_defaults
        .iter()
        .map(|(priority, details)| GroupDefaults {
            priority: *priority,
            max_byte_size: details.max_byte_size,
        })
        .collect::<Vec<GroupDefaults>>();
    let data = StoreData {
        byte_size: store.byte_size,
        max_byte_size: store.max_byte_size,
        msg_count: store.id_to_group_map.len(),
        group_count: store.groups_map.len(),
        groups,
        group_defaults,
    };
    Reply::OkWData(data)
}

const ROUTE: &'static str = "GET /api/store";
pub fn http_handle(data: Data<AppData>) -> HttpResponse {
    http_route_hit_log::<()>(ROUTE, None);
    http_reply("GET /api/store", handle(data))
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>) {
    http_route_hit_log::<()>(STORE_GET, None);
    ws_reply_with(ctx, STORE_GET)(handle(data));
}
