use crate::{
    api::{
        get_optional_max_byte_size, http_reply, update_config,
        ws::{command::STORE_PUT, Websocket},
        ws_reply_with, Reply, lock_or_exit, http_route_hit_log, FileManager
    },
    AppData,
};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use actix_web_actors::ws::WebsocketContext;
use log::error;
use msg_store::store::StoreDefaults;
use serde::Deserialize;
use serde_json::Value;
use std::process::exit;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    max_byte_size: Option<u32>,
}

pub fn validate_body(value: Value) -> Result<Body, Reply<()>> {
    let max_byte_size = match get_optional_max_byte_size(&value) {
        Ok(max_byte_size) => Ok(max_byte_size),
        Err(message) => Err(Reply::BadRequest(message)),
    }?;
    Ok(Body { max_byte_size })
}

pub fn handle(data: Data<AppData>, body: Body) -> Reply<()> {
    
    let (prune_count, pruned_uuids) = {
        let mut store = lock_or_exit(&data.store);        
        let max_byte_size = body.max_byte_size;
        store.max_byte_size = max_byte_size;
        let defaults = StoreDefaults {
            max_byte_size,
        };
        match store.update_store_defaults(&defaults) {
            Ok((_bytes_removed, _groups_removed, msgs_removed)) => (msgs_removed.len() as u32, msgs_removed),
            Err(error) => {
                error!("ERROR_CODE: 80d8a9bd-8739-47b1-89ac-34dd30e8cef5. Could not update store defaults: {}", error);
                exit(1);
            }
        }
    };
    if let Some(file_manager) = &data.file_manager {
        for uuid in pruned_uuids {
            FileManager::del(file_manager, &uuid);
        }
    }    
    {
        let mut stats = lock_or_exit(&data.stats);
        stats.pruned += prune_count;
    }
    {
        let mut config = lock_or_exit(&data.configuration);
        config.max_byte_size = body.max_byte_size;
        if let Err(error) = update_config(&mut config, &data.configuration_path) {
            error!("ERROR_CODE: 9f1f42ea-f259-44e6-9da1-38e3c9f24d6c. Could not update config file: {}", error);
            exit(1);
        }
    }    
    Reply::Ok
}

const ROUTE: &'static str = "PUT /api/store";
pub fn http_handle(data: Data<AppData>, body: Json<Value>) -> HttpResponse {
    http_route_hit_log(ROUTE, Some(body.clone()));
    let reply = match validate_body(body.into_inner()) {
        Ok(body) => handle(data, body),
        Err(reply) => reply,
    };
    http_reply(ROUTE, reply)
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, body: Value) {
    http_route_hit_log(STORE_PUT, Some(body.clone()));
    let reply = match validate_body(body) {
        Ok(body) => handle(data, body),
        Err(reply) => reply,
    };
    ws_reply_with(ctx, STORE_PUT)(reply);
}
