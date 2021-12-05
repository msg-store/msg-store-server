use actix_web::{
    HttpResponse,
    web::{
        Data,
        Json
    }
};
use crate::{
    api::update_config,
    AppData,
    config::GroupConfig,
    fmt_result
};
use msg_store::store::{
    GroupDefaults
};
use serde::{
    Deserialize, 
    Serialize
};
use std::borrow::BorrowMut;

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    priority: i32,
    max_byte_size: Option<i32>
}

pub fn post(data: Data<AppData>, body: Json<Body>) -> HttpResponse {
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

    let defaults = GroupDefaults {
        max_byte_size: body.max_byte_size
    };

    if let Err(_error) = store.update_group_defaults(body.priority, &defaults) {
        return HttpResponse::InternalServerError().finish();
    }

    let mk_group_config = || -> GroupConfig {
        GroupConfig {
            priority: body.priority,
            max_byte_size: body.max_byte_size
        }
    };

    if let Some(groups) = config.groups.borrow_mut() {
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
    } else {
        config.groups = Some(vec![
            mk_group_config()
        ]);
    }

    if let Err(_error) = update_config(&config, &data.config_location) {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
  
}
