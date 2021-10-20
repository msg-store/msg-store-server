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
    DbGaurd,
    StoreGaurd,
    fmt_result,
    api::{
        msg::{
            post::post_msg,
            delete::delete_msg
        }
    }
};
use msg_store::store::{
    GroupId,
    MsgByteSize
};
use serde::{
    Deserialize, 
    Serialize
};
use std::{
    path::PathBuf
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    max_byte_size: Option<MsgByteSize>, // default = null
    prune: Option<bool>, // default = true
    update_config: Option<bool> // default = true
}

pub fn update_store(
    db: &mut DbGaurd,
    store: &mut StoreGaurd,
    config: &mut ConfigGaurd,
    config_location: &PathBuf,
    body: &Body) -> Result<(), String> {
    let prune = match body.prune {
        Some(prune) => prune,
        None => true
    };
    let update_config = match body.update_config {
        Some(update) => update,
        None => true
    };
    store.max_byte_size = body.max_byte_size;
    if prune {
        // TODO: check if a pruning needs a'fixen
        let id = fmt_result!(post_msg(db, store, &GroupId::MAX, ""))?;
        fmt_result!(delete_msg(db, store, &id))?;
    }
    if update_config {
        config.max_byte_size = body.max_byte_size;
        fmt_result!(config.update_config_file(config_location))?;
    }
    Ok(())
}

pub fn update(data: Data<AppData>, body: Json<Body>) -> HttpResponse {
    let mut db = match fmt_result!(data.db.try_lock()) {
        Ok(db) => db,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
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
    match fmt_result!(update_store(&mut db, &mut store, &mut config, &data.config_location, &body.into_inner())) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_error) => {
            HttpResponse::InternalServerError().finish()
        }
    }
    
}
