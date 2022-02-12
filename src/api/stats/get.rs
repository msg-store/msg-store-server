use crate::AppData;
use crate::api::lower::error_codes::log_err;
use crate::api::lower::stats::get::handle;
use actix_web::{web::Data, HttpResponse};
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
