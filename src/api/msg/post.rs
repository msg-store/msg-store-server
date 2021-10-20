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
    database::{
        leveldb::{
            MsgData
        }
    },
    fmt_result
};
use msg_store::store::{
    GroupId,
    MsgId,
    MsgByteSize,
    Msg,
    insert,
    delete
};
use serde::{
    Deserialize, 
    Serialize
};
use std::{
    convert::TryInto
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    priority: GroupId,
    msg: String
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { id: MsgId }
}

pub fn post_msg(db: &mut DbGaurd, store: &mut StoreGaurd, priority: &GroupId, msg: &str) -> Result<MsgId, String> {
    let msg_byte_size: MsgByteSize = fmt_result!(msg.len().try_into())?;
    let msg_data = MsgData {
        priority: *priority,
        msg_byte_size
    };
    let store_msg = Msg {
        byte_size: msg_byte_size,
        priority: *priority
    };
    let insert_result = fmt_result!(insert(store, &store_msg))?;
    let ids_removed = insert_result.ids_removed;
    for id in ids_removed.iter() {
        db.delete(&id)?;
    }
    let id = insert_result.id;
    match fmt_result!(db.put(&id, &msg_data, msg)) {
        Ok(_) => Ok(id),
        Err(error) => {
            fmt_result!(delete(store, &id))?;
            Err(error)
        }
    }
}

pub fn post(data: Data<AppData>, body: Json<Body>) -> HttpResponse {
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
    let id = match fmt_result!(post_msg(&mut db, &mut store, &body.priority, &body.msg)) {
        Ok(id) => id,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };

    HttpResponse::Ok().json(Reply::Ok { id })
}