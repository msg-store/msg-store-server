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
    fmt_result,
    api::{
        msg::{
            delete::{
                delete_msg
            }
        }
    }
};
use msg_store::store::{
    GroupId,
    MsgId
};
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    priority: GroupId
}

pub fn delete_group(db: &mut DbGaurd, store: &mut StoreGaurd, info: &Info) -> Result<(), String> {
    let group = match store.groups_map.get(&info.priority) {
        Some(group) => group,
        None => {
            return Ok(())
        }
    };
    let msg_ids: Vec<MsgId> = group.msgs_map.keys().map(|key| *key).collect();
    for id in msg_ids.iter() {
        delete_msg(db, store, id)?;
    }
    Ok(())
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
    match fmt_result!(delete_group(&mut db, &mut store, &info.into_inner())) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_error) => HttpResponse::InternalServerError().finish()
    }
}
