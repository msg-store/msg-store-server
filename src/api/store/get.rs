use actix_web::{
    HttpResponse,
    web::{
        Data
    }
};
use crate::{
    AppData,
    fmt_result
};
use msg_store::store::{
    GroupId,
    MsgByteSize
};
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct StoreData {
    byte_size: MsgByteSize,
    max_byte_size: Option<MsgByteSize>,
    msg_count: usize,
    group_count: usize,
    groups: Vec<GroupData>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupData {
    priority: GroupId,
    byte_size: MsgByteSize,
    max_byte_size: Option<MsgByteSize>,
    msg_count: usize    
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { data: StoreData }
}

pub fn get(data: Data<AppData>) -> HttpResponse {
    let store = match fmt_result!(data.store.try_lock()) {
        Ok(db) => db,
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
    let data = StoreData {
        byte_size: store.byte_size,
        max_byte_size: store.max_byte_size,
        msg_count: store.id_to_group_map.len(),
        group_count: store.groups_map.len(),
        groups
    };
    HttpResponse::Ok().json(Reply::Ok{ data })
}
