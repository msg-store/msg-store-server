use std::{
    path::{
        PathBuf
    }, 
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
use msg_store::store::{
    Store,
    generate_store
};

mod api;
mod args;
mod config;
mod database;
mod error;

use config::{
    StoreConfig
};

pub type DbGaurd<'a> = MutexGuard<'a, database::leveldb::Db>;
pub type StoreGaurd<'a> = MutexGuard<'a, Store>;
pub type ConfigGaurd<'a> = MutexGuard<'a, StoreConfig>;

pub struct AppData {
    pub db: Mutex<database::leveldb::Db>,
    pub store: Mutex<Store>,
    pub config_location: PathBuf,
    pub config: Mutex<StoreConfig>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    
    let args = args::Args::default();

    let store_config = StoreConfig::open(&args.config_location);

    let db_location: PathBuf = {
        match &args.database_location {
            Some(location) => location.to_path_buf(),
            None => match &store_config.location {
                Some(location) => location.to_path_buf(),
                None => {
                    panic!("Database location was not set.");
                }
            }
        }
    };

    let mut db = database::leveldb::Db::open(&db_location).unwrap();
    let mut store = generate_store();

    db.setup(&mut store, &store_config).expect("Could not setup database.");

    let app_data = Data::new(AppData {
        db: Mutex::new(db),
        store: Mutex::new(store),
        config_location: args.config_location,
        config: Mutex::new(store_config)
    });

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(middleware::Logger::default())
            .app_data(app_data.clone())
            
            .route("/api/group", web::delete().to(api::group::delete::delete))
            .route("/api/group", web::get().to(api::group::get::get))
            
            .route("/api/group_defaults", web::delete().to(api::group_defaults::delete::delete))
            .route("/api/group_defaults", web::get().to(api::group_defaults::get::get))
            .route("/api/group_defaults", web::post().to(api::group_defaults::post::post))
            
            .route("/api/msg", web::get().to(api::msg::get::get))
            .route("/api/msg", web::delete().to(api::msg::delete::delete))
            .route("/api/msg", web::post().to(api::msg::post::post))
            .route("/api/msg", web::put().to(api::msg::put::update))
            
            .route("/api/store", web::get().to(api::store::get::get))
            .route("/api/store", web::put().to(api::store::put::update))
    })
    // start http server on 127.0.0.1:8080
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
