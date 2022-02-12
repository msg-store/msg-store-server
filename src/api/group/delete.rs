use crate::{
    api::{
        from_value_prop_required, http_reply,
        // ws::{command::GROUP_DELETE, Websocket},
        // ws_reply_with, 
        Reply, lock_or_exit, http_route_hit_log,
        lower::file_storage::rm_from_file_storage
    },
    AppData,
};
use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use actix_web_actors::ws::WebsocketContext;
use log::error;
use msg_store::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    process::exit,
    sync::Arc
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Info {
    priority: u32,
}

pub fn handle(data: Data<AppData>, info: Info) -> Reply<()> {
    let list = {
        let store = lock_or_exit(&data.store);
        // get list of messages to remove
        let list = if let Some(group) = store.groups_map.get(&info.priority) {
            group
                .msgs_map
                .keys()
                .map(|uuid| { uuid.clone() })
                .collect::<Vec<Arc<Uuid>>>()
        } else {
            return Reply::Ok;
        };
        list
    };
    let mut deleted_count = 0;
    for uuid in list.iter() {
        {
            let mut store = lock_or_exit(&data.store);
            if let Err(error) = store.del(uuid.clone()) {
                error!("ERROR_CODE: 919e9188-e8cc-45a3-844c-ba3b4914e771. Could not delete msg from store: {}", error);
                exit(1);
            }
        }
        {
            let mut db = lock_or_exit(&data.db);
            if let Err(error) = db.del(uuid.clone()) {
                error!("ERROR_CODE: 62539cba-bfa6-4421-bf72-aa491c14f963. Could not delete msg from database: {}", error);
                exit(1);
            }
        }
        if let Some(file_storage_mutex) = &data.file_storage {
            let mut file_storage = lock_or_exit(file_storage_mutex);
            if let Err(error_code) = rm_from_file_storage(&mut file_storage, uuid) {
                error!("ERROR_CODE: {}", error_code);
                exit(1);
            }
        }
        deleted_count += 1;
    }
    
    let mut stats = lock_or_exit(&data.stats);
    stats.deleted += deleted_count;
    Reply::Ok
}

const ROUTE: &'static str = "DEL /api/group";
pub fn handle_http(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    http_route_hit_log(ROUTE, Some(info.clone()));
    http_reply(ROUTE, handle(data, info.into_inner()))
}

// pub fn handle_ws(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, info: Value) {
//     http_route_hit_log(GROUP_DELETE, Some(info.clone()));
//     let mut reply = ws_reply_with(ctx, GROUP_DELETE);
//     let priority = match from_value_prop_required(&info, "priority", "number") {
//         Ok(priority) => priority,
//         Err(message) => {
//             reply(Reply::BadRequest(message));
//             return;
//         }
//     };
//     reply(handle(data, Info { priority }));
// }
