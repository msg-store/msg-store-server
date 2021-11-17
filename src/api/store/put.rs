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
use msg_store::store::StoreDefaults;

use serde::{
    Deserialize, 
    Serialize
};
use std::{
    path::PathBuf
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    max_byte_size: Option<i32>
}

pub fn update_store(
    store: &mut StoreGaurd,
    config: &mut ConfigGaurd,
    config_location: &Option<PathBuf>,
    body: &Body) -> Result<(), String> {
    store.max_byte_size = body.max_byte_size;
    let defaults = StoreDefaults { max_byte_size: body.max_byte_size };
    store.update_store_defaults(&defaults);
    if let Some(config_location) = config_location {
        config.max_byte_size = body.max_byte_size;
        fmt_result!(config.update_config_file(config_location))?;
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
