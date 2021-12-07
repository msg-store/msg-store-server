use actix_web::{
    HttpResponse,
    web::{
        Data,
        Query
    }
};
use crate::AppData;
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    priority: Option<u32>,
    include_msg_data: Option<bool>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Msg {
    uuid: String,
    byte_size: u32
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    priority: u32,
    byte_size: u32,
    max_byte_size: Option<u32>,
    msg_count: u32,
    messages: Vec<Msg>
}


// #[derive(Debug, Deserialize, Serialize)]
// pub struct DefaultsWithPriority {
//     priority: u32,
//     max_byte_size: Option<u32>
// }

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { data: Option<Group> },
    OkMany { data: Vec<Group> }
}

pub fn get(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    if let Some(priority) = info.priority {
        if let Some(group) = store.groups_map.get(&priority) {
            let group = Group {
                priority: priority.clone(),
                byte_size: group.byte_size,
                max_byte_size: group.max_byte_size,
                msg_count: group.msgs_map.len() as u32,
                messages: match info.include_msg_data {
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
        } else {
            HttpResponse::Ok().json(Reply::Ok{data: None})
        }
    } else {
        let data = store.groups_map.iter().map(|(priority, group)| {
            Group {
                priority: priority.clone(),
                byte_size: group.byte_size,
                max_byte_size: group.max_byte_size,
                msg_count: group.msgs_map.len() as u32,
                messages: match info.include_msg_data {
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
