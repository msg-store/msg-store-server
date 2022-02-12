use crate::{
    api::{
        get_optional_max_byte_size, http_reply, update_config,
        // ws::{command::STORE_PUT, Websocket},
        // ws_reply_with, 
        Reply, lock_or_exit, http_route_hit_log,
        lower::file_storage::rm_from_file_storage
    },
    AppData,
};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
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
    if let Some(file_storage_mutex) = &data.file_storage {
        let mut file_storage = lock_or_exit(&file_storage_mutex);
        for uuid in pruned_uuids {
            if let Err(error_code) = rm_from_file_storage(&mut file_storage, &uuid) {
                error!("ERROR_CODE: {}", error_code);
                exit(1);
            }
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
