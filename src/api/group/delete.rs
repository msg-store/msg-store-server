use actix_web::{
    HttpResponse,
    web::{
        Data,
        Query
    }
};
use crate::AppData;
use msg_store::Uuid;
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    priority: i32
}

pub fn delete(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let list = {
        let store = match data.store.try_lock() {
            Ok(store) => store,
            Err(_error) => {
                return HttpResponse::InternalServerError().finish();
            }
        };
    
        // get list of messages to remove
        let list = if let Some(group) = store.groups_map.get(&info.priority) {
            group.msgs_map.keys().map(|uuid| -> Uuid { uuid.clone() }).collect::<Vec<Uuid>>()
        } else {
            return HttpResponse::Ok().finish();
        };
        list
    };

    for uuid in list.iter() {
        let mut store = match data.store.try_lock() {
            Ok(store) => store,
            Err(_error) => {
                return HttpResponse::InternalServerError().finish();
            }
        };
        if let Err(_error) = store.del(uuid) {
            return HttpResponse::InternalServerError().finish();
        }
    }
    HttpResponse::Ok().finish()
    
}
