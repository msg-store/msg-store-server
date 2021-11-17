use std::{
    sync::{
        Mutex,
        MutexGuard
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
mod error;
mod init;

use config::{
    StoreConfig
};

use init::{
    Store,
    init
};

pub type StoreGaurd<'a> = MutexGuard<'a, Store>;
pub type ConfigGaurd<'a> = MutexGuard<'a, StoreConfig>;

pub struct AppData {
    pub store: Mutex<Store>,
    // pub config_location: Option<PathBuf>,
    pub config: Mutex<StoreConfig>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    
    // let args = args::Args::default();
    // let (store, store_config, config_location) = StoreConfig::open(&args.config_location);
    

    // let _db_location: PathBuf = {
    //     match &args.database_location {
    //         Some(location) => location.to_path_buf(),
    //         None => match &store_config.location {
    //             Some(location) => location.to_path_buf(),
    //             None => {
    //                 panic!("Database location was not set.");
    //             }
    //         }
    //     }
    // };


    let (store, store_config) = init();

    


    let app_data = Data::new(AppData {
        store: Mutex::new(store),
        // config_location,
        config: Mutex::new(store_config)
    });

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .app_data(app_data.clone())
            
            .route("/api/group", web::get().to(api::group::get::get))
            
            .route("/api/group_defaults", web::delete().to(api::group_defaults::delete::delete))
            .route("/api/group_defaults", web::get().to(api::group_defaults::get::get))
            .route("/api/group_defaults", web::post().to(api::group_defaults::post::post))
            
            .route("/api/msg", web::get().to(api::msg::get::get))
            .route("/api/msg", web::delete().to(api::msg::delete::delete))
            .route("/api/msg", web::post().to(api::msg::post::post))
            
            .route("/api/store", web::get().to(api::store::get::get))
            .route("/api/store", web::put().to(api::store::put::update))
    })
    // start http server on 127.0.0.1:8080
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
