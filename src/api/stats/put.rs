use crate::{
    api::{
        from_value_prop, get_optional_number, http_reply,
        // ws::{command::STATS_PUT, Websocket},
        // ws_reply_with, 
        Reply, lock_or_exit, http_route_hit_log,
        lower::stats::Stats
    },
    AppData
};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::MutexGuard;

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

pub fn get_value(value: Value) -> Result<Body, Reply<Stats>> {
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

pub fn handle(data: Data<AppData>, body: Value) -> Reply<Stats> {
    let body = match get_value(body) {
        Ok(body) => body,
        Err(reply) => return reply,
    };
    let add_to = |body: Body, stats: &mut MutexGuard<Stats>| {
        if let Some(inserted) = body.inserted {
            stats.inserted += inserted;
        }
        if let Some(deleted) = body.deleted {
            stats.deleted += deleted;
        }
        if let Some(pruned) = body.pruned {
            stats.pruned += pruned;
        }
    };
    let replace_current = |body: Body, stats: &mut MutexGuard<Stats>| {
        if let Some(inserted) = body.inserted {
            stats.inserted = inserted;
        }
        if let Some(deleted) = body.deleted {
            stats.deleted = deleted;
        }
        if let Some(pruned) = body.pruned {
            stats.pruned = pruned;
        }
    };
    let stats = {
        let mut stats = lock_or_exit(&data.stats);
        let old_stats = stats.clone();
        if let Some(add) = body.add {
            if add {
                add_to(body, &mut stats);
            } else {
                replace_current(body, &mut stats);
            }
        } else {
            replace_current(body, &mut stats);
        }
        old_stats
    };
    Reply::OkWData(stats)
}

const ROUTE: &'static str = "PUT /api/stats";
pub fn http_handle(data: Data<AppData>, body: Json<Value>) -> HttpResponse {
    http_route_hit_log(ROUTE, Some(body.clone()));
    http_reply(ROUTE, handle(data, body.into_inner()))
}

// pub fn ws_handle(ctx: &mut ws::WebsocketContext<Websocket>, data: Data<AppData>, body: Value) {
//     http_route_hit_log(STATS_PUT, Some(body.clone()));
//     ws_reply_with(ctx, STATS_PUT)(handle(data, body));
// }
