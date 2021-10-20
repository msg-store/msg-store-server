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
    MsgByteSize
};
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    priority: GroupId
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupDefaults {
    max_byte_size: Option<MsgByteSize>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { data: Option<GroupDefaults> }
}

pub fn get(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let store = match fmt_result!(data.store.try_lock()) {
        Ok(db) => db,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    match store.group_defaults.get(&info.priority) {
        Some(defaults) => {
            let group_defaults = GroupDefaults{ max_byte_size: defaults.max_byte_size };
            HttpResponse::Ok().json(Reply::Ok{data: Some(group_defaults)})
        },
        None => {
            HttpResponse::Ok().json(Reply::Ok{data: None})
        }
    }
}
