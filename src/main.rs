use actix_web::{
    middleware,
    web::{self, Data},
    App, HttpServer,
};
use std::{path::PathBuf, sync::Mutex};

mod api;
mod config;
mod init;

use config::StoreConfig;

use init::{init, Store};

// pub type StoreGaurd<'a> = MutexGuard<'a, Store>;
// pub type ConfigGaurd<'a> = MutexGuard<'a, StoreConfig>;

pub struct AppData {
    pub store: Mutex<Store>,
    pub config_location: Option<PathBuf>,
    pub config: Mutex<StoreConfig>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");

    let init_result = init();

    let app_data = Data::new(AppData {
        store: Mutex::new(init_result.store),
        config_location: init_result.config_location,
        config: Mutex::new(init_result.store_config),
    });

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .app_data(app_data.clone())
            .route("/api/export", web::get().to(api::export::http_handle))
            .route(
                "/api/group",
                web::delete().to(api::group::delete::handle_http),
            )
            .route("/api/group", web::get().to(api::group::get::http_handle))
            .route(
                "/api/group-defaults",
                web::delete().to(api::group_defaults::delete::http_handle),
            )
            .route(
                "/api/group-defaults",
                web::get().to(api::group_defaults::get::http_handle),
            )
            .route(
                "/api/group-defaults",
                web::post().to(api::group_defaults::post::http_handle),
            )
            .route("/api/msg", web::get().to(api::msg::get::http_handle))
            .route("/api/msg", web::delete().to(api::msg::delete::http_handle))
            .route("/api/msg", web::post().to(api::msg::post::http_handle))
            .route(
                "/api/stats",
                web::delete().to(api::stats::delete::http_handle),
            )
            .route("/api/stats", web::get().to(api::stats::get::http_handle))
            .route("/api/stats", web::put().to(api::stats::put::http_handle))
            .route("/api/store", web::get().to(api::store::get::http_handle))
            .route("/api/store", web::put().to(api::store::put::http_handle))
            .service(web::resource("/ws").route(web::get().to(api::ws::ws_index)))
    })
    // start http server on 127.0.0.1:8080
    .bind(init_result.host)?
    .run()
    .await
}
