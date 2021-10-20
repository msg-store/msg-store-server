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
    fmt_err_msg,
    fmt_result
};
use msg_store::store::{
    GroupId,
    MsgId,
    get_next,
    get_next_from_group,
    msg_exists
};
use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct MsgData {
    id: MsgId,
    priority: GroupId,
    msg: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    id: Option<MsgId>,
    priority: Option<MsgId>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Reply {
    Ok { data: Option<MsgData> }
}

pub fn get_msg(db: &mut DbGaurd, store: &mut StoreGaurd, info: &Info) -> Result<Option<MsgData>, String> {
    let id = match info.id {
        Some(id) => match msg_exists(store, &id)? {
            true => id,
            false => {
                return Ok(None);
            }
        },
        None => match info.priority {
            Some(priority) => match fmt_result!(get_next_from_group(store, &priority))? {
                Some(id) => id,
                None => {
                    return Ok(None);
                }
            },
            None => match fmt_result!(get_next(store))? {
                Some(id) => id,
                None => {
                    return Ok(None);
                }
            }
        }
    };
    let msg = match fmt_result!(db.get(&id))? {
        Some(msg) => msg,
        None => {
            return Err(fmt_err_msg!("Sync Error: Id found in store, but not in leveldb."));
        }
    };
    let priority = match store.id_to_group_map.get(&id) {
        Some(priority) => *priority,
        None => {
            return Err(fmt_err_msg!("Sync Error: Id found in store, but not in store.id_to_group_map."));
        }
    };
    let msg_data = MsgData {
        id,
        priority,
        msg
    };
    Ok(Some(msg_data))
}

pub fn get(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
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
    match fmt_result!(get_msg(&mut db, &mut store, &info.into_inner())) {
        Ok(data) => {
            HttpResponse::Ok().json(Reply::Ok{ data })
        },
        Err(_error) => {
            HttpResponse::InternalServerError().finish()
        }
    }
}