use crate::{
    api::{
        from_value_prop, get_optional_number, http_reply,
        ws::{command::STATS_PUT, Websocket},
        ws_reply_with, Reply,
    },
    AppData, Store,
};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{process::exit, sync::MutexGuard};

#[derive(Debug, Deserialize, Serialize)]
pub struct StatsProps {
    pub inserted: Option<u32>,
    pub deleted: Option<u32>,
    pub pruned: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    add: Option<bool>,
    inserted: Option<u32>,
    deleted: Option<u32>,
    pruned: Option<u32>,
}

pub fn get_value(value: Value) -> Result<Body, Reply<()>> {
    let add = match from_value_prop::<bool, _>(&value, "add", "boolean") {
        Ok(value) => value,
        Err(_error) => {
            return Err(Reply::BadRequest(
                "Invalid parameter: /data/add".to_string(),
            ))
        }
    };
    let inserted = match get_optional_number(&value, "inserted") {
        Ok(value) => value,
        Err(_error) => {
            return Err(Reply::BadRequest(
                "Invalid parameter: /data/inserted".to_string(),
            ))
        }
    };
    let deleted = match get_optional_number(&value, "deleted") {
        Ok(value) => value,
        Err(_error) => {
            return Err(Reply::BadRequest(
                "Invalid parameter: /data/deleted".to_string(),
            ))
        }
    };
    let pruned = match get_optional_number(&value, "pruned") {
        Ok(value) => value,
        Err(_error) => {
            return Err(Reply::BadRequest(
                "Invalid parameter: /data/pruned".to_string(),
            ))
        }
    };
    Ok(Body {
        add,
        inserted,
        deleted,
        pruned,
    })
}

pub fn handle(data: Data<AppData>, body: Value) -> Reply<()> {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            exit(1);
        }
    };
    let body = match get_value(body) {
        Ok(body) => body,
        Err(reply) => return reply,
    };
    let add_to = |body: Body, store: &mut MutexGuard<Store>| {
        if let Some(inserted) = body.inserted {
            store.msgs_inserted += inserted;
        }
        if let Some(deleted) = body.deleted {
            store.msgs_deleted += deleted;
        }
        if let Some(pruned) = body.pruned {
            store.msgs_pruned += pruned;
        }
    };
    let replace_current = |body: Body, store: &mut MutexGuard<Store>| {
        if let Some(inserted) = body.inserted {
            store.msgs_inserted = inserted;
        }
        if let Some(deleted) = body.deleted {
            store.msgs_deleted = deleted;
        }
        if let Some(pruned) = body.pruned {
            store.msgs_pruned = pruned;
        }
    };
    if let Some(add) = body.add {
        if add {
            add_to(body, &mut store);
        } else {
            replace_current(body, &mut store);
        }
    } else {
        replace_current(body, &mut store);
    }
    Reply::Ok
}

pub fn http_handle(data: Data<AppData>, body: Json<Value>) -> HttpResponse {
    http_reply(handle(data, body.into_inner()))
}

pub fn ws_handle(ctx: &mut ws::WebsocketContext<Websocket>, data: Data<AppData>, body: Value) {
    ws_reply_with(ctx, STATS_PUT)(handle(data, body));
}
