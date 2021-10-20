use actix_web::{
    HttpResponse,
    web::{
        Data,
        Query
    }
};
use crate::{
    AppData,
    DbGaurd,
    StoreGaurd,
    fmt_result
};
use msg_store::store::{
    MsgId,
    delete as delete_msg_in_store
};
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    id: MsgId
}

pub fn delete_msg(db: &mut DbGaurd, store: &mut StoreGaurd, id: &MsgId) -> Result<(), String> {
    fmt_result!(db.delete(id))?;
    fmt_result!(delete_msg_in_store(store, id))
}

pub fn delete(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let mut db = match fmt_result!(data.db.try_lock()) {
        Ok(db) => db,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut store = match fmt_result!(data.store.try_lock()) {
        Ok(db) => db,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    match fmt_result!(delete_msg(&mut db, &mut store, &info.id)) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_error) => HttpResponse::InternalServerError().finish()
    }
}