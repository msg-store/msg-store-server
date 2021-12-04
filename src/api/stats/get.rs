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
pub struct Stats {
    inserted: i32,
    deleted: i32,
    pruned: i32
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { data: Stats }
}

pub fn get(data: Data<AppData>) -> HttpResponse {
    let store = match fmt_result!(data.store.try_lock()) {
        Ok(db) => db,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(Reply::Ok{ data: Stats {
        inserted: store.msgs_inserted,
        deleted: store.msgs_deleted,
        pruned: store.msgs_burned
    } })
}
