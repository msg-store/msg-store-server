use crate::{
    api::{
        get_required_uuid, http_bad_request, http_reply, validate_uuid_string,
        ws::{command::MSG_DELETE, Websocket},
        ws_reply_with, Reply, lock_or_exit, http_route_hit_log, FileManager
    },
    AppData,
};
use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use actix_web_actors::ws;
use log::{
    error,
    // debug
};
use msg_store::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    process::exit,
    sync::Arc
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Info {
    uuid: String,
}

pub fn handle(data: Data<AppData>, uuid: Arc<Uuid>) -> Reply<()> {
    {
        let mut store = lock_or_exit(&data.store);
        if let Err(error) = store.del(uuid.clone()) {
            error!("ERROR_CODE: b2a60844-8e3b-4ca9-bccb-e13ada4fadd7. Could not remove msg from store: {}", error);
            exit(1);
        }
    }
    {
        let mut db = lock_or_exit(&data.db);
        if let Err(error) = db.del(uuid.clone()) {
            error!("ERROR_CODE: 37897fc1-578d-45a1-824b-7b1a1519e6ef. Could not removed msg from database: {}", error);
            exit(1);
        }
    }
    {
        if let Some(file_manager) = &data.file_manager {
            FileManager::del(file_manager, uuid);
        }
    }
    {
        let mut stats = lock_or_exit(&data.stats);
        stats.deleted += 1;
    }
    Reply::Ok
}

const ROUTE: &'static str = "DEL /api/msg";
pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    http_route_hit_log(ROUTE, Some(info.clone()));
    let uuid = match validate_uuid_string(info.into_inner().uuid) {
        Ok(uuid) => uuid,
        Err(message) => {
            return http_bad_request(ROUTE, message);
        }
    };
    // debug!("uuid: {}", uuid.to_string());
    http_reply(ROUTE, handle(data, uuid))
}

pub fn ws_handle(ctx: &mut ws::WebsocketContext<Websocket>, data: Data<AppData>, info: Value) {
    http_route_hit_log(MSG_DELETE, Some(info.clone()));
    let mut reply = ws_reply_with(ctx, MSG_DELETE);
    let uuid = match get_required_uuid(&info) {
        Ok(uuid) => uuid,
        Err(message) => {
            return reply(Reply::BadRequest(format!("/data/{}", message)));
        }
    };
    reply(handle(data, uuid));
}
