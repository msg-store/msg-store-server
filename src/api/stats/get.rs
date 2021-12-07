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
    priority: u32,
    max_byte_size: Option<u32>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupData {
    priority: u32,
    byte_size: u32,
    max_byte_size: Option<u32>,
    msg_count: usize
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Stats {
    pub inserted: u32,
    pub deleted: u32,
    pub pruned: u32
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok(Stats)
}

pub fn get(data: Data<AppData>) -> HttpResponse {
    let store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    HttpResponse::Ok().json(Reply::Ok(Stats {
        inserted: store.msgs_inserted,
        deleted: store.msgs_deleted,
        pruned: store.msgs_pruned
    }))
}
