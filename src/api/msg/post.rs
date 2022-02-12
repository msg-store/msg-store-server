use crate::{
    api::{
        http_route_hit_log,
        lower::{
            error_codes,
            msg::add::handle
        }
    },
    AppData,
};
use actix_web::{
    web::{ Data,Payload },
    HttpResponse,
};
use log::info;
use serde::{Deserialize, Serialize};
use std::process::exit;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReturnBody {
    uuid: String,
}

const STATUS_400: &'static [&'static str] = &[
    error_codes::COULD_NOT_GET_CHUNK_FROM_PAYLOAD,
    error_codes::COULD_NOT_PARSE_CHUNK,
    error_codes::MISSING_HEADERS,
    error_codes::MALFORMED_HEADERS,
    error_codes::INVALID_PRIORITY,
    error_codes::MISSING_PRIORITY,
    error_codes::INVALID_BYTESIZE_OVERRIDE,
    error_codes::MISSING_BYTESIZE_OVERRIDE
];
const STATUS_403: &'static [&'static str] = &[
    error_codes::FILE_STORAGE_NOT_CONFIGURED
];
const STATUS_409: &'static [&'static str] = &[
    error_codes::MSG_EXCEEDES_STORE_MAX,
    error_codes::MSG_EXCEEDES_GROUP_MAX,
    error_codes::MSG_LACKS_PRIORITY
];

const ROUTE: &'static str = "POST /api/msg";
pub async fn http_handle(data: Data<AppData>, body: Payload) -> HttpResponse {
    http_route_hit_log::<()>(ROUTE, None);
    match handle(&data.store, &data.file_storage, &data.stats, &data.db, &mut body.into_inner()).await {
        Ok(uuid) => HttpResponse::Ok().json(ReturnBody { uuid: uuid.to_string() }),
        Err(error_code) => {
            if STATUS_400.contains(&error_code) {
                info!("{} 400 {}", ROUTE, error_code);
                HttpResponse::BadRequest().body(error_code)
            } else if STATUS_403.contains(&error_code) {
                info!("{} 403 {}", ROUTE, error_code);
                HttpResponse::Forbidden().body(error_code)
            } else if STATUS_409.contains(&error_code) {
                info!("{} 409 {}", ROUTE, error_code);
                HttpResponse::Conflict().body(error_code)
            } else {
                error_codes::log_err(error_code, file!(), line!(), "");
                exit(1)
            }
        }
    }
}
