use crate::AppData;
use crate::api::lower::error_codes;
use crate::api::lower::group_defaults::set::handle;
use actix_web::HttpResponse;
use actix_web::web::{Data, Json};
use log::info;
use serde::{Deserialize, Serialize};
use std::process::exit;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    priority: u32,
    max_byte_size: Option<u32>,
}

const ROUTE: &'static str = "POST /api/group-defaults";
pub fn http_handle(data: Data<AppData>, info: Json<Info>) -> HttpResponse {
    {
        let max_byte_size_string = if let Some(max_byte_size) = info.max_byte_size {
            max_byte_size.to_string()
        } else {
            "N/A".to_string()
        };
        info!("{} priority: {}, max byte size: {}", ROUTE, info.priority, max_byte_size_string);
    }
    let result = handle(
        &data.store, 
        &data.db, 
        &data.file_storage, 
        &data.stats, 
        &data.configuration, 
        &data.configuration_path, 
        info.priority, 
        info.max_byte_size);
    if let Err(error_code) = result {
        error_codes::log_err(error_code, file!(), line!(), "");
        exit(1);
    }
    info!("{} 200", ROUTE);
    HttpResponse::Ok().finish()
}
