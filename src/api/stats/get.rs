use actix_web::{
    HttpResponse,
    web::{
        Data
    }
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
pub struct Stats {
    pub inserted: i32,
    pub deleted: i32,
    pub pruned: i32
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { data: Stats }
}

pub fn get(data: Data<AppData>) -> HttpResponse {
    let store = match data.store.try_lock() {
        Ok(store) => store,
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
