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
    priority: Option<i32>,
    include_msg_data: Option<bool>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Msg {
    uuid: String,
    byte_size: i32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    priority: i32,
    byte_size: i32,
    max_byte_size: Option<i32>,
    msg_count: i32,
    msgs: Vec<Msg>
}


#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultsWithPriority {
    priority: i32,
    max_byte_size: Option<i32>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { data: Option<Group> },
    OkMany { data: Vec<Group> }
}

pub fn get(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let store = match fmt_result!(data.store.try_lock()) {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    match &info.priority {
        Some(priority) => match store.groups_map.get(priority) {
            Some(group) => {
                let group = Group {
                    priority: priority.clone(),
                    byte_size: group.byte_size,
                    max_byte_size: group.max_byte_size,
                    msg_count: group.msgs_map.len() as i32,
                    msgs: match info.include_msg_data {
                        Some(include_msg_data) => match include_msg_data {
                            true => group.msgs_map.iter().map(|(uuid, byte_size)| { 
                                Msg{uuid: uuid.to_string(), byte_size: byte_size.clone()}
                            }).collect::<Vec<Msg>>(),
                            false => vec![]
                        },
                        None => vec![]
                    } 
                };
                HttpResponse::Ok().json(Reply::Ok{data: Some(group)})
            },
            None => {
                HttpResponse::Ok().json(Reply::Ok{data: None})
            }
        },
        None => {
            let data = store.groups_map.iter().map(|(priority, group)| {
                Group {
                    priority: priority.clone(),
                    byte_size: group.byte_size,
                    max_byte_size: group.max_byte_size,
                    msg_count: group.msgs_map.len() as i32,
                    msgs: match info.include_msg_data {
                        Some(include_msg_data) => match include_msg_data {
                            true => group.msgs_map.iter().map(|(uuid, byte_size)| { 
                                Msg{uuid: uuid.to_string(), byte_size: byte_size.clone()}
                            }).collect::<Vec<Msg>>(),
                            false => vec![]
                        },
                        None => vec![]
                    }
                }
            }).collect::<Vec<Group>>();
            HttpResponse::Ok().json(Reply::OkMany{data})
        }
    }
    
}
