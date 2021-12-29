use crate::{
    api::{get_optional_number, get_required_string},
    AppData,
};
use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use actix_web_actors::ws::WebsocketContext;
use msg_store::{GetOptions, Uuid};
use serde::{Deserialize, Serialize};
use serde_json::{to_string, Value};
use std::{
    fs::OpenOptions, io::Write, ops::Bound::Included, path::PathBuf, process::exit, str::FromStr,
};

use super::{
    append_null, prepend_data_str,
    ws::{command::EXPORT, Websocket},
    ws_reply_with, Reply,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct StoredPacket {
    pub uuid: String,
    pub msg: String,
}

#[derive(Debug, Deserialize, Serialize)]
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
        let store = match data.store.try_lock() {
            Ok(store) => store,
            Err(_error) => {
                exit(1);
            }
        };
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
        Err(_error) => {
            exit(1);
        }
    };
    let mut file = match OpenOptions::new()
        .append(true)
        .create_new(true)
        .open(file_path)
    {
        Ok(file) => file,
        Err(_error) => {
            exit(1);
        }
    };
    for uuid in list {
        let mut store = match data.store.try_lock() {
            Ok(store) => store,
            Err(_error) => {
                exit(1);
            }
        };
        let msg_option = match store.get(GetOptions::default().uuid(uuid)) {
            Ok(msg) => msg,
            Err(_error) => {
                exit(1);
            }
        };
        let stored_packet = if let Some(stored_packet) = msg_option {
            stored_packet
        } else {
            continue;
        };
        let transformed_stored_packet = StoredPacket {
            uuid: stored_packet.uuid.to_string(),
            msg: stored_packet.msg,
        };
        let mut packet_string = match to_string(&transformed_stored_packet) {
            Ok(packet_string) => packet_string,
            Err(_error) => {
                exit(1);
            }
        };
        packet_string.push_str("\n");
        if let Err(_error) = file.write_all(packet_string.as_bytes()) {
            exit(1);
        };
        if let Err(_error) = store.del(&uuid) {
            exit(1);
        }
    }
    Reply::Ok
}

pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    handle(data, info.into_inner());
    HttpResponse::Ok().finish()
}

pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, info: Value) {
    let mut reply = ws_reply_with(ctx, EXPORT);
    let info = match get_info(&info) {
        Ok(info) => info,
        Err(message) => {
            return reply(Reply::BadRequest(message));
        }
    };
    reply(handle(data, info));
}
