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
    uuid: String
}

pub fn delete(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let uuid = match Uuid::from_string(&info.uuid) {
        Ok(uuid) => uuid,
        Err(_error) => {
            return HttpResponse::BadRequest().content_type("text/plain").body("Query deserialize error: Invalid UUID");
        }
    };
    if let Err(_error) = store.del(&uuid) {
        HttpResponse::InternalServerError().finish()
    } else {
        HttpResponse::Ok().finish()
    }
}