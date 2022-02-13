use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use crate::AppData;
use msg_store::api::error_codes::log_err;
use msg_store::api::stats::set::handle;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;
use std::process::exit;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    add: Option<bool>,
    inserted: Option<u32>,
    deleted: Option<u32>,
    pruned: Option<u32>,
}
impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(self))
    }
}

const ROUTE: &'static str = "PUT /api/stats";
pub fn http_handle(data: Data<AppData>, info: Json<Info>) -> HttpResponse {
    info!("{} {}", ROUTE, info);
    let add = if let Some(add) = info.add {
        add
    } else {
        false
    };
    match handle(&data.stats, add, info.inserted, info.deleted, info.pruned) {
        Ok(stats) => {
            info!("{} 200 {}", ROUTE, stats);
            HttpResponse::Ok().json(stats)
        },
        Err(error_code) => {
            log_err(error_code, file!(), line!(), "");
            exit(1);
        }
    }
}
