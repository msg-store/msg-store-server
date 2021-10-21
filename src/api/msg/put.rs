use actix_web::{
    HttpResponse,
    web::{
        Data,
        Json
    }
};
use crate::{
    AppData,
    DbGaurd,
    StoreGaurd,
    fmt_result,
    database::{
        leveldb::{
            MsgData
        }
    }
};
use msg_store::store::{
    GroupId,
    MsgId,
    MoveResult,
    mv
};
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    id: MsgId,
    new_priority: GroupId
}

pub fn update_msg(db: &mut DbGaurd, store: &mut StoreGaurd, id: &MsgId, new_priority: &GroupId) -> Result<(), String> {
    match mv(store, id, new_priority) {
        MoveResult::Ok{ insert_result, msg_data } => {
            for id in insert_result.ids_removed.iter() {
                db.delete(id).unwrap();
            }
            let data = MsgData {
                priority: msg_data.priority,
                msg_byte_size: msg_data.byte_size
            };
            db.update(id, &data).unwrap();
            Ok(())
        },
        MoveResult::Err{ error } => Err(error),
        MoveResult::InsertionError{ insert_result, error } => {
            for id in insert_result.ids_removed.iter() {
                db.delete(id).unwrap();
            }
            Err(error)
        },
        MoveResult::ReinsertionError{ insert_error, reinsert_error } => {
            Err(format!("ReinsertionError: {}\n{}", insert_error, reinsert_error))
        }
    }
}

pub fn update(data: Data<AppData>, info: Json<Info>) -> HttpResponse {
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
    match fmt_result!(update_msg(&mut db, &mut store, &info.id, &info.new_priority)) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(error) => {
            println!("{}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}