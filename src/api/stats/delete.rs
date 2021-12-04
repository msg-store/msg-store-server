use actix_web::{
    HttpResponse,
    web::{
        Data
    }
};
use crate::{
    AppData,
    StoreGaurd,
    fmt_result,
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

pub fn reset_stats(store: &mut StoreGaurd) -> Stats {
    let stats = Stats {
        inserted: store.msgs_inserted,
        deleted: store.msgs_deleted,
        pruned: store.msgs_burned
    };
    store.msgs_inserted = 0;
    store.msgs_deleted = 0;
    store.msgs_burned = 0;
    stats
}

pub fn delete(data: Data<AppData>) -> HttpResponse {
    let mut store = match fmt_result!(data.store.try_lock()) {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let data = reset_stats(&mut store);
    HttpResponse::Ok().json(Reply::Ok { data })
}