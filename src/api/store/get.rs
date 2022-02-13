use actix_web::{web::Data, HttpResponse};
use crate::AppData;
use msg_store::api::store::get::handle;
use msg_store::api::error_codes::log_err;
use log::info;
use std::process::exit;

const ROUTE: &'static str = "GET /api/store";
pub fn http_handle(data: Data<AppData>) -> HttpResponse {
    info!("{}", ROUTE);
    match handle(&data.store) {
        Ok(store_data) => {
            info!("{} 200", ROUTE);
            HttpResponse::Ok().json(store_data)
        },
        Err(error_code) => {
            log_err(error_code, file!(), line!(), "");
            exit(1);
        }
    }
}
