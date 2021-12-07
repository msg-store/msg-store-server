use actix_web::{
    HttpResponse,
    web::{
        Data,
        Query
    }
};
use crate::{api::update_config, AppData, config::GroupConfig};

use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    priority: u32
}

pub enum ErrorCode {
    StoreLockingError,
    ConfigLockingError,
    ConfigUpdateError
}
impl ErrorCode {
    pub fn to_int(&self) -> u32 {
        match self {
            Self::StoreLockingError => 1,
            Self::ConfigLockingError => 2,
            Self::ConfigUpdateError => 3
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Reply {
    Error { code: u32, message: String }
}
impl Reply {
    pub fn store_locking_error() -> Reply {
        Reply::Error { code: ErrorCode::StoreLockingError.to_int(), message: "Internal server error".to_string() }
    }
    pub fn config_locking_error() -> Reply {
        Reply::Error { code: ErrorCode::ConfigLockingError.to_int(), message: "Internal server error".to_string() }
    }
    pub fn update_error() -> Reply {
        Reply::Error { code: ErrorCode::ConfigUpdateError.to_int(), message: "The store was updated, but the changes were not saved due to an error.".to_string() }
    }
}


pub fn delete(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().json(Reply::store_locking_error());
        }
    };
    store.delete_group_defaults(info.priority);
    let mut config = match data.config.try_lock() {
        Ok(config) => config,
        Err(_error) => {
            return HttpResponse::InternalServerError().json(Reply::config_locking_error());
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
    if let Err(_error) = update_config(&config, &data.config_location) {
        return HttpResponse::InternalServerError().json(Reply::update_error());
    }
    HttpResponse::Ok().finish()
}
