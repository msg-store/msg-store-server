use actix_web::{
    HttpResponse,
    web::{
        Data,
        Query
    }
};
use crate::{
    AppData,
    fmt_result
};
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    priority: Option<i32>
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct GroupDefaults {
    priority: i32,
    max_byte_size: Option<i32>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { data: Option<GroupDefaults> },
    OkMany { data: Vec<GroupDefaults> }
}

pub fn get(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let store = match fmt_result!(data.store.try_lock()) {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    match &info.priority {
        Some(priority) => match store.group_defaults.get(priority) {
            Some(defaults) => {
                let group_defaults = GroupDefaults {
                    priority: priority.clone(),
                    max_byte_size: defaults.max_byte_size
                };
                HttpResponse::Ok().json(Reply::Ok{data: Some(group_defaults)})
            },
            None => {
                HttpResponse::Ok().json(Reply::Ok{data: None})
            }
        },
        None => {
            let data = store.group_defaults.iter().map(|(priority, defaults)| {
                GroupDefaults {
                    priority: priority.clone(),
                    max_byte_size: defaults.max_byte_size
                }
            }).collect::<Vec<GroupDefaults>>();
            HttpResponse::Ok().json(Reply::OkMany{data})
        }
    }
    
}
