use actix_web::HttpResponse;
use actix_web::web::{Data, Query};
use crate::AppData;
use msg_store::api::error_codes;
use msg_store::api::group_defaults::get::handle;
use log::info;
use serde::{Deserialize, Serialize};
use std::process::exit;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Info {
    priority: Option<u32>,
}

const ROUTE: &'static str = "GET /api/group-defaults";
pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let result = handle(&data.store, info.priority);
    match result {
        Ok(groups) => {
            info!("{} 200", ROUTE);
            HttpResponse::Ok().json(groups)
        },
        Err(error_code) => {
            error_codes::log_err(error_code, file!(), line!(), "");
            exit(1)
        }
    }
}
