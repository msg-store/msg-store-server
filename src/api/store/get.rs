use actix_web::{
    HttpResponse,
    web::Data
};
use crate::AppData;

use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupDefaults {
    priority: i32,
    max_byte_size: Option<i32>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupData {
    priority: i32,
    byte_size: i32,
    max_byte_size: Option<i32>,
    msg_count: usize
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StoreData {
    byte_size: i32,
    max_byte_size: Option<i32>,
    msg_count: usize,
    group_count: usize,
    groups: Vec<GroupData>,
    group_defaults: Vec<GroupDefaults>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { data: StoreData }
}

pub fn get(data: Data<AppData>) -> HttpResponse {
    let store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let groups = store.groups_map.iter().map(|(priority, group)| {
        GroupData {
            priority: *priority,
            byte_size: group.byte_size,
            max_byte_size: group.max_byte_size,
            msg_count: group.msgs_map.len()
        }
    }).collect::<Vec<GroupData>>();
    let group_defaults = store.group_defaults.iter().map(|(priority, details)| {
        GroupDefaults {
            priority: *priority,
            max_byte_size: details.max_byte_size
        }
    }).collect::<Vec<GroupDefaults>>();
    let data = StoreData {
        byte_size: store.byte_size,
        max_byte_size: store.max_byte_size,
        msg_count: store.id_to_group_map.len(),
        group_count: store.groups_map.len(),
        groups,
        group_defaults
    };
    HttpResponse::Ok().json(Reply::Ok{ data })
}
