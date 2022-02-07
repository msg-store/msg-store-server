use crate::{
    api::{
        get_optional_number, get_required_string, http_route_hit_log, format_log_complete, http_reply,
        ws::command::EXPORT
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
use serde_json::{to_string, Value};
use std::{
    fs::OpenOptions, 
    io::Write, 
    ops::Bound::Included, 
    path::PathBuf, 
    process::exit, 
    str::FromStr,
};

// TODO: FIX: Export only exports msgs stored in the database, not the filesytem

use super::{
    append_null, prepend_data_str,
    ws::Websocket,
    ws_reply_with, Reply, lock_or_exit,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct StoredPacket {
    pub uuid: String,
    pub msg: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    output: String,
    priority: Option<u32>,
    range_start: Option<u32>,
    range_end: Option<u32>,
}

pub fn get_info(value: &Value) -> Result<Info, String> {
    let output = match get_required_string(&value, "output") {
        Ok(output) => Ok(output),
        Err(message) => Err(prepend_data_str(message)),
    }?;
    let priority = match get_optional_number(&value, "priority") {
        Ok(value) => Ok(value),
        Err(message) => Err(prepend_data_str(message)),
    }?;
    let range_start = match get_optional_number(&value, "range_start") {
        Ok(value_result) => match value_result {
            Some(value) => Ok(Some(value)),
            None => match get_optional_number(&value, "rangeStart") {
                Ok(value) => Ok(value),
                Err(message) => Err(append_null(prepend_data_str(message))),
            },
        },
        Err(message) => Err(append_null(prepend_data_str(message))),
    }?;
    let range_end = match get_optional_number(&value, "range_end") {
        Ok(value_result) => match value_result {
            Some(value) => Ok(Some(value)),
            None => match get_optional_number(&value, "rangeEnd") {
                Ok(value) => Ok(value),
                Err(message) => Err(append_null(prepend_data_str(message))),
            },
        },
        Err(message) => Err(append_null(prepend_data_str(message))),
    }?;
    Ok(Info {
        output,
        priority,
        range_start,
        range_end,
    })
}

pub fn handle(data: Data<AppData>, info: Info) -> Reply<()> {
    let list = {
        let store = lock_or_exit(&data.store);
        let list: Vec<Uuid> = if let Some(priority) = info.priority {
            if let Some(group) = store.groups_map.get(&priority) {
                group
                    .msgs_map
                    .keys()
                    .map(|uuid| uuid.clone())
                    .collect::<Vec<Uuid>>()
            } else {
                vec![]
            }
        } else if let Some(start) = info.range_start {
            let end = if let Some(end) = info.range_end {
                end
            } else {
                u32::MAX
            };
            let mut list = vec![];
            for (_priority, group) in store.groups_map.range((Included(&start), Included(&end))) {
                list.append(
                    &mut group
                        .msgs_map
                        .keys()
                        .map(|uuid| uuid.clone())
                        .collect::<Vec<Uuid>>(),
                )
            }
            list
        } else if let Some(end) = info.range_end {
            let start = u32::MIN;
            let mut list = vec![];
            for (_priority, group) in store.groups_map.range((Included(&start), Included(&end))) {
                list.append(
                    &mut group
                        .msgs_map
                        .keys()
                        .map(|uuid| uuid.clone())
                        .collect::<Vec<Uuid>>(),
                )
            }
            list
        } else {
            let mut list = vec![];
            for group in store.groups_map.values() {
                list.append(
                    &mut group
                        .msgs_map
                        .keys()
                        .map(|uuid| uuid.clone())
                        .collect::<Vec<Uuid>>(),
                )
            }
            list
        };
        list
    };
    let file_path = match PathBuf::from_str(&info.output) {
        Ok(dir_path) => dir_path,
        Err(error) => {
            error!("ERROR_CODE: ab65abce-5a25-415b-991a-7a540242d185. Could not parse file path: {}", error.to_string());
            exit(1);
        }
    };
    let mut file = match OpenOptions::new()
        .append(true)
        .create_new(true)
        .open(file_path)
    {
        Ok(file) => file,
        Err(error) => {
            error!("ERROR_CODE: 5d935776-a61f-4b07-9a0f-0d4c7e68280e. Could not open file: {}", error.to_string());
            exit(1);
        }
    };
    let mut deleted_count = 0;
    for uuid in list {
        let uuid = {
            let store = lock_or_exit(&data.store);
            match store.get(Some(uuid), None, false) {
                Ok(uuid) => uuid,
                Err(error) => {
                    error!("ERROR_CODE: eed54108-ed28-4809-97dc-3559930a1a4d. Could not get data from store: {}", error.to_string());
                    exit(1);
                }
            }            
        };
        let uuid = if let Some(uuid) = uuid {
            uuid
        } else {
            continue;
        };        
        let msg = {
            let mut db = lock_or_exit(&data.db);
            match db.get(uuid) {
                Ok(msg) => msg,
                Err(error) => {
                    error!("ERROR_CODE: 244c3834-6d38-40f4-901b-58e7444a091a. Could not get msg from database: {}", error.to_string());
                    exit(1);
                }
            }
        };
        let msg = match String::from_utf8(msg.to_vec()) {
            Ok(msg) => msg,
            Err(_error) => {
                error!("ERROR_CODE: 7862fbdf-0bdb-45ae-b3fd-71a053102451. Could not get msg from bytes");
                exit(1);
            }
        };
        let transformed_stored_packet = StoredPacket {
            uuid: uuid.to_string(),
            msg,
        };
        let mut packet_string = match to_string(&transformed_stored_packet) {
            Ok(packet_string) => packet_string,
            Err(error) => {
                error!("ERROR_CODE: 175f94cb-3553-461d-9447-7dd3e3ff9b87. Could not convert data to string: {}", error.to_string());
                exit(1);
            }
        };
        packet_string.push_str("\n");
        if let Err(error) = file.write_all(packet_string.as_bytes()) {
            error!("ERROR_CODE: 7471d35e-a5a2-4bd6-bab1-6017092d97b5. Could not write to file: {}", error.to_string());
            exit(1);
        };
        {
            let mut store = lock_or_exit(&data.store);
            if let Err(error) = store.del(&uuid) {
                error!("ERROR_CODE: c70634e2-f090-43b1-883a-ccf8e62fbc30. Could not get mutex lock on store: {}", error.to_string());
                exit(1);
            }
        }
        {
            let mut db = lock_or_exit(&data.db);
            if let Err(error) = db.del(uuid) {
                error!("ERROR_CODE: 4a25b256-753c-43c6-b564-35448d5ad9de. Could not get mutex lock on database: {}", error.to_string());
                exit(1);
            }
        }        
        deleted_count += 1;
    }
    {
        let mut stats = lock_or_exit(&data.stats);
        stats.deleted += deleted_count;
    }    
    Reply::Ok
}

const ROUTE: &'static str = "/api/export";

pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    http_route_hit_log(ROUTE, Some(info.clone()));
    http_reply(ROUTE, handle(data, info.into_inner()))
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, info: Value) {
    http_route_hit_log(EXPORT, Some(info.clone()));
    let mut reply = ws_reply_with(ctx, EXPORT);
    let info = match get_info(&info) {
        Ok(info) => info,
        Err(message) => {
            format_log_complete::<()>(ROUTE, 400, None);
            return reply(Reply::BadRequest(message));
        }
    };
    reply(handle(data, info));
}
