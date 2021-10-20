use actix_web::{
    HttpResponse,
    web::{
        Data,
        Query
    }
};
use crate::{AppData, config::GroupConfig, fmt_result};
use msg_store::store::{
    GroupId
};
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    priority: GroupId,
    update_group: Option<bool>, // default = true
    update_config: Option<bool> // default = true
}

pub fn delete(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let mut store = match fmt_result!(data.store.try_lock()) {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    store.group_defaults.remove(&info.priority);
    let update_group = match info.update_group {
        Some(update_group) => update_group,
        None => true
    };
    if update_group {
        if let Some(mut group) = store.groups_map.get_mut(&info.priority) {
            group.max_byte_size = None;
        }
    }
    let update_config = match info.update_group {
        Some(update_group) => update_group,
        None => true
    };
    if update_config {
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
        if let Err(_error) = fmt_result!(config.update_config_file(&data.config_location)) {
            return HttpResponse::InternalServerError().finish();
        }
    }    
    HttpResponse::Ok().finish()
}
