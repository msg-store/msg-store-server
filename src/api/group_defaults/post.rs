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
    api::{
        msg::{
            delete::delete_msg,
            post::post_msg
        }
    }, 
    config::GroupConfig,
    fmt_result
};
use msg_store::store::{
    GroupId,
    GroupDefaults,
    MsgByteSize
};
use serde::{
    Deserialize, 
    Serialize
};
use std::{
    borrow::BorrowMut,
    path::PathBuf
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    priority: GroupId,
    max_byte_size: Option<MsgByteSize>,
    prune: Option<bool>, // default = true
    update_config: Option<bool> // default = true
}

pub fn post_group_defaults(
    db: &mut DbGaurd,
    store: &mut StoreGaurd, 
    config: &mut ConfigGaurd, 
    config_location: &PathBuf, 
    body: &Body) -> Result<(), String> {
    let update_config = match body.update_config {
        Some(update) => update,
        None => true
    };
    let prune = match body.prune {
        Some(prune) => prune,
        None => true
    };
    if let Some(group) = store.groups_map.get_mut(&body.priority) {
        group.max_byte_size = body.max_byte_size;
    }
    if let Some(mut defaults) = store.group_defaults.get_mut(&body.priority) {
        defaults.max_byte_size = body.max_byte_size;
    } else {
        let defaults = GroupDefaults {
            max_byte_size: body.max_byte_size
        };
        store.group_defaults.insert(body.priority, defaults);
    }
    if update_config {
        let mk_group_config = || -> GroupConfig {
            GroupConfig {
                priority: body.priority,
                max_byte_size: body.max_byte_size
            }
        };
        match config.groups.borrow_mut() {
            Some(groups) => {
                let mut group_index: Option<usize> = None;
                for i in 0..groups.len() {
                    let group = match groups.get(i) {
                        Some(group) => group,
                        None => {
                            continue;
                        }
                    };
                    if body.priority == group.priority {
                        group_index = Some(i);
                        break;
                    }
                }
                if let Some(index) = group_index {
                    if let Some(group) = groups.get_mut(index) {
                        group.max_byte_size = body.max_byte_size;
                    } else {
                        groups.push(mk_group_config());
                    }
                } else {
                    groups.push(mk_group_config());
                }
                groups.sort();
            },
            None => {
                config.groups = Some(vec![
                    mk_group_config()
                ]);
            }
        };        
        fmt_result!(config.update_config_file(&config_location))?;
    }
    if prune {
        let id = fmt_result!(post_msg(db, store, &body.priority, ""))?;
        fmt_result!(delete_msg(db, store, &id))?;
    }
    Ok(())
}

pub fn post(data: Data<AppData>, body: Json<Body>) -> HttpResponse {
    let mut db = match fmt_result!(data.db.try_lock()) {
        Ok(db) => db,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish()
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
            return HttpResponse::InternalServerError().finish()
        }
    };
    match fmt_result!(post_group_defaults(&mut db, &mut store, &mut config, &data.config_location, &body.into_inner())) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_error) => HttpResponse::InternalServerError().finish()
    }    
}
