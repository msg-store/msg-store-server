use actix_web::{
    HttpResponse,
    web::{
        Data
    }
};
use crate::{
    AppData,
    api::stats::get::Stats
};

use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { data: Stats }
}

pub fn delete(data: Data<AppData>) -> HttpResponse {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let data = Stats {
        inserted: store.msgs_inserted,
        deleted: store.msgs_deleted,
        pruned: store.msgs_pruned
    };
    store.msgs_inserted = 0;
    store.msgs_deleted = 0;
    store.msgs_pruned = 0;
    HttpResponse::Ok().json(Reply::Ok { data })
}