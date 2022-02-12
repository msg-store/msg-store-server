use crate::AppData;
use crate::api::lower::error_codes::log_err;
use crate::api::lower::store::set::handle;
use actix_web::HttpResponse;
use actix_web::web::{Data, Json};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;
use std::process::exit;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    max_byte_size: Option<u32>,
}

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(self))
    }
}

const ROUTE: &'static str = "PUT /api/store";
pub fn http_handle(data: Data<AppData>, info: Json<Info>) -> HttpResponse {
    info!("{} {}", ROUTE, info);
    let result = handle(
        &data.store, 
        &data.file_storage, 
        &data.stats, 
        &data.configuration, 
        &data.configuration_path, 
        info.max_byte_size);
    if let Err(error_code) = result {
        log_err(error_code, file!(), line!(), "");
        exit(1);
    }
    HttpResponse::Ok().finish()
}
