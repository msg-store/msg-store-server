use actix_web::{
    HttpResponse,
    web::{
        Data,
        Query
    }
};
use crate::{AppData, config::GroupConfig, fmt_result};

use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    priority: i32
}

pub fn delete(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let mut store = match fmt_result!(data.store.try_lock()) {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    store.delete_group_defaults(info.priority);
    let mut config = match fmt_result!(data.config.try_lock()) {
        Ok(config) => config,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish()
        }
    };
    let groups = match &config.groups {
        Some(groups) => groups,
        None => {
            return HttpResponse::Ok().finish();
        }
    };
    let new_groups: Vec<GroupConfig> = groups.iter().filter(|group| {
        if group.priority != info.priority {
            true
        } else {
            false
        }
    }).map(|group| group.clone()).collect();
    config.groups = Some(new_groups);
    if let Some(config_location) = &data.config_location {
        if let Err(_error) = fmt_result!(config.update_config_file(&config_location)) {
            return HttpResponse::InternalServerError().finish();
        }
    }    
    HttpResponse::Ok().finish()
}
