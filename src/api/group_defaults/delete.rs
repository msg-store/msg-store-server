use crate::AppData;
use crate::api::lower::error_codes;
use crate::api::lower::group_defaults::rm::handle;
use actix_web::HttpResponse;
use actix_web::web::{Data, Query};
use log::info;
use serde::{Deserialize, Serialize};
use std::process::exit;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Info {
    priority: u32,
}

const ROUTE: &'static str = "DEL /api/group-defaults";
pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    info!("{} priority: {}", ROUTE, info.priority);
    let result = handle(
        &data.store, 
        &data.configuration, 
        &data.configuration_path, 
        info.priority);
    if let Err(error_code) = result {
        error_codes::log_err(error_code, file!(), line!(), "");
        exit(1);
    }
    info!("{} 200", ROUTE);
    HttpResponse::Ok().finish()
}
