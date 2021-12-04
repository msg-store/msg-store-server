use actix_web::{
    HttpResponse,
    web::{
        Data,
        Json
    }
};
use crate::{
    AppData,
    ConfigGaurd,
    StoreGaurd,
    fmt_result
};

use serde::{
    Deserialize, 
    Serialize
};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    pub inserted: Option<i32>,
    pub deleted: Option<i32>,
    pub pruned: Option<i32>
}

pub fn update_store(
    store: &mut StoreGaurd,
    config: &mut ConfigGaurd,
    config_location: &Option<PathBuf>,
    body: &Body) -> Result<(), String> {
    if let Some(inserted) = body.inserted {
        store.msgs_inserted = inserted;
    }
    if let Some(deleted) = body.deleted {
        store.msgs_deleted = deleted;
    }
    if let Some(pruned) = body.pruned {
        store.msgs_burned = pruned;
    }
    Ok(())
}

pub fn update(data: Data<AppData>, body: Json<Body>) -> HttpResponse {
    let mut store = match fmt_result!(data.store.try_lock()) {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut config = match fmt_result!(data.config.try_lock()) {
        Ok(config) => config,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    match fmt_result!(update_store(&mut store, &mut config, &data.config_location, &body.into_inner())) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_error) => {
            HttpResponse::InternalServerError().finish()
        }
    }
    
}
