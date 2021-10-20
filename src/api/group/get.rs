use actix_web::{
    HttpResponse,
    web::{
        Data,
        Query
    }
};
use crate::{
    AppData,
    fmt_result
};
use msg_store::store::{
    GroupId,
    MsgByteSize,
    MsgId
};
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    priority: GroupId,
    include_msg_data: Option<bool> // default = false
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MsgData {
    id: MsgId,
    byte_size: MsgByteSize
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupData {
    priority: GroupId,
    byte_size: MsgByteSize,
    max_byte_size: Option<MsgByteSize>,
    msg_count: usize,
    msg_data: Option<Vec<MsgData>>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { data: Option<GroupData> }
}

pub fn get(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let store = match fmt_result!(data.store.try_lock()) {
        Ok(db) => db,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    match store.groups_map.get(&info.priority) {
        Some(group) => {
            let mut msg_data_vec = None;
            if let Some(get_msg_data) = info.include_msg_data {
                if get_msg_data {
                    let mut msgs = vec![];
                    for (id, byte_size) in group.msgs_map.iter() {
                        let msg_data = MsgData {
                            id: *id,
                            byte_size: *byte_size
                        };
                        msgs.push(msg_data);
                    }
                    msg_data_vec = Some(msgs);
                }
            }
            let group_data = GroupData {
                priority: info.priority,
                byte_size: group.byte_size,
                max_byte_size: group.max_byte_size,
                msg_count: group.msgs_map.len(),
                msg_data: msg_data_vec
            };
            HttpResponse::Ok().json(Reply::Ok{data: Some(group_data)})
        },
        None => HttpResponse::Ok().json(Reply::Ok{ data: None })
    }
}