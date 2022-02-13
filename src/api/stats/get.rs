use actix_web::{web::Data, HttpResponse};
use crate::AppData;
use msg_store::api::error_codes::log_err;
use msg_store::api::stats::get::handle;
use log::info;
use std::process::exit;

const ROUTE: &'static str = "GET /api/stats";
pub fn http_handle(data: Data<AppData>) -> HttpResponse {
    info!("{}", ROUTE);
    match handle(&data.stats) {
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
