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
pub struct StatsProps {
    pub inserted: Option<u32>,
    pub deleted: Option<u32>,
    pub pruned: Option<u32>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Body {
    Replace { inserted: Option<u32>, deleted: Option<u32>, pruned: Option<u32> },
    Add { inserted: Option<u32>, deleted: Option<u32>, pruned: Option<u32> }
}

pub fn update(data: Data<AppData>, body: Json<Body>) -> HttpResponse {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    match body.0 {
        Body::Replace { inserted, deleted, pruned } => {
            if let Some(inserted) = inserted {
                store.msgs_inserted = inserted;
            }
            if let Some(deleted) = deleted {
                store.msgs_deleted = deleted;
            }
            if let Some(pruned) = pruned {
                store.msgs_pruned = pruned;
            }
        },
        Body::Add { inserted, deleted, pruned } => {
            if let Some(inserted) = inserted {
                store.msgs_inserted += inserted;
            }
            if let Some(deleted) = deleted {
                store.msgs_deleted += deleted;
            }
            if let Some(pruned) = pruned {
                store.msgs_pruned += pruned;
            }
        }
    }

    HttpResponse::Ok().finish()    
}
