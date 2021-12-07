use std::{
    path::PathBuf,
    sync::{
        Mutex,
        // MutexGuard
    }};
use actix_web::{
    middleware, 
    web::{
        self,
        Data
    }, 
    App, 
    HttpServer
};

mod api;
mod config;
mod init;

use config::{
    StoreConfig
};

use init::{
    Store,
    init
};

// pub type StoreGaurd<'a> = MutexGuard<'a, Store>;
// pub type ConfigGaurd<'a> = MutexGuard<'a, StoreConfig>;

pub struct AppData {
    pub store: Mutex<Store>,
    pub config_location: Option<PathBuf>,
    pub config: Mutex<StoreConfig>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");

    let init_result = init();

    let app_data = Data::new(AppData {
        store: Mutex::new(init_result.store),
        config_location: init_result.config_location,
        config: Mutex::new(init_result.store_config)
    });

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .app_data(app_data.clone())

            .route("/api/export", web::get().to(api::export::get))
            
            .route("/api/group", web::delete().to(api::group::delete::delete))
            .route("/api/group", web::get().to(api::group::get::get))
            
            .route("/api/group-defaults", web::delete().to(api::group_defaults::delete::delete))
            .route("/api/group-defaults", web::get().to(api::group_defaults::get::get))
            .route("/api/group-defaults", web::post().to(api::group_defaults::post::post))
            
            .route("/api/msg", web::get().to(api::msg::get::get))
            .route("/api/msg", web::delete().to(api::msg::delete::delete))
            .route("/api/msg", web::post().to(api::msg::post::post))
            
            .route("/api/stats", web::delete().to(api::stats::delete::delete))
            .route("/api/stats", web::get().to(api::stats::get::get))
            .route("/api/stats", web::put().to(api::stats::put::update))

            .route("/api/store", web::get().to(api::store::get::get))
            .route("/api/store", web::put().to(api::store::put::update))
    })
    // start http server on 127.0.0.1:8080
    .bind(init_result.host)?
    .run()
    .await
}
