use crate::{
    api::{
        get_optional_max_byte_size, get_required_priority, http_reply, update_config,
        ws::{command::GROUP_DEFAULTS_POST, Websocket},
        ws_reply_with, Reply, lock_or_exit, http_route_hit_log,
        lower::file_storage::rm_from_file_storage
    },
    config::GroupConfig,
    AppData,
};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use actix_web_actors::ws::WebsocketContext;
use log::error;
use msg_store::store::GroupDefaults;
use serde_json::Value;
use std::{borrow::BorrowMut, process::exit};

pub struct Body {
    priority: u32,
    max_byte_size: Option<u32>,
}

pub fn validate_body(body: Value) -> Result<Body, String> {
    let priority = get_required_priority(&body)?;
    let max_byte_size = get_optional_max_byte_size(&body)?;
    Ok(Body {
        priority,
        max_byte_size,
    })
}

pub fn handle(data: Data<AppData>, body: Value) -> Reply<()> {
    let body = match validate_body(body) {
        Ok(body) => body,
        Err(message) => return Reply::BadRequest(message),
    };
    let defaults = GroupDefaults {
        max_byte_size: body.max_byte_size,
    };
    let (pruned_count, msgs_removed) = {
        let mut store = lock_or_exit(&data.store);
        match store.update_group_defaults(body.priority, &defaults) {
            Ok((_bytes_removed, msgs_removed)) => (msgs_removed.len() as u32, msgs_removed),
            Err(error) => {
                error!("ERROR_CODE: caff3867-7e32-4945-aa80-1527acdb295c. Could not update group defaults in store: {}", error);
                exit(1);
            }
        }
    };
    for uuid in msgs_removed.into_iter() {
        {
            let mut db = lock_or_exit(&data.db);
            if let Err(error) = db.del(uuid.clone()) {
                error!("ERROR_CODE: 34ed09d2-7d65-4709-8aba-997a42564634. Could not prune msg in database: {}", error);
                exit(1);
            }
        }
        if let Some(file_storage_mutex) = &data.file_storage {
            let mut file_storage = lock_or_exit(file_storage_mutex);
            if let Err(error_code) = rm_from_file_storage(&mut file_storage, &uuid) {
                error!("ERROR_CODE: {}", error_code);
                exit(1);
            }
        }
    }
    
    {
        let mut stats = lock_or_exit(&data.stats);
        stats.pruned += pruned_count;
    }
    let mk_group_config = || -> GroupConfig {
        GroupConfig {
            priority: body.priority,
            max_byte_size: body.max_byte_size,
        }
    };
    {
        let mut config = lock_or_exit(&data.configuration);
        if let Some(groups) = config.groups.borrow_mut() {
            let mut group_index: Option<usize> = None;
            for i in 0..groups.len() {
                let group = match groups.get(i) {
                    Some(group) => group,
                    None => {
                        continue;
                    }
                };
                if body.priority == group.priority {
                    group_index = Some(i);
                    break;
                }
            }
            if let Some(index) = group_index {
                if let Some(group) = groups.get_mut(index) {
                    group.max_byte_size = body.max_byte_size;
                } else {
                    groups.push(mk_group_config());
                }
            } else {
                groups.push(mk_group_config());
            }
            groups.sort();
        } else {
            config.groups = Some(vec![mk_group_config()]);
        }
        if let Err(error) = update_config(&config, &data.configuration_path) {
            error!("ERROR_CODE: 88a850f5-c310-4c95-9f38-6535322beb7d. Could not update config file: {}", error);
            exit(1);
        }
    }    
    Reply::Ok
}

const ROUTE: &'static str = "POST /api/group-defaults";
pub fn http_handle(data: Data<AppData>, body: Json<Value>) -> HttpResponse {
    http_route_hit_log(ROUTE, Some(body.clone()));
    http_reply(ROUTE, handle(data, body.into_inner()))
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, body: Value) {
    http_route_hit_log(GROUP_DEFAULTS_POST, Some(body.clone()));
    (ws_reply_with(ctx, GROUP_DEFAULTS_POST)(handle(data, body)));
}
