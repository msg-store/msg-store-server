use actix_web::{
    HttpResponse,
    web::{
        Data,
        Query
    }
};
use crate::{
    AppData,
    StoreGaurd,
    fmt_result
};
use msg_store::Uuid;
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    uuid: String
}

pub fn delete_msg(store: &mut StoreGaurd, uuid: &Uuid) -> Result<(), String> {
    store.del(uuid)
}

pub fn delete(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let mut store = match fmt_result!(data.store.try_lock()) {
        Ok(db) => db,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    match fmt_result!(delete_msg(&mut store, &Uuid::from_string(&info.uuid))) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_error) => HttpResponse::InternalServerError().finish()
    }
}