use actix_web::{
    HttpResponse,
    web::{
        Data,
        Json
    }
};
use crate::AppData;

use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    pub inserted: Option<i32>,
    pub deleted: Option<i32>,
    pub pruned: Option<i32>
}

pub fn update(data: Data<AppData>, body: Json<Body>) -> HttpResponse {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    if let Some(inserted) = body.inserted {
        store.msgs_inserted = inserted;
    }
    if let Some(deleted) = body.deleted {
        store.msgs_deleted = deleted;
    }
    if let Some(pruned) = body.pruned {
        store.msgs_pruned = pruned;
    }
    HttpResponse::Ok().finish()    
}
