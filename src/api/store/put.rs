use actix_web::{
    HttpResponse,
    web::{
        Data,
        Json
    }
};
use crate::{
    api::update_config,
    AppData
};
use msg_store::store::StoreDefaults;

use serde::{
    Deserialize, 
    Serialize
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    max_byte_size: Option<u32>
}

pub fn update(data: Data<AppData>, body: Json<Body>) -> HttpResponse {
    let mut store = match data.store.try_lock() {
        Ok(store) => store,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    let mut config = match data.config.try_lock() {
        Ok(config) => config,
        Err(_error) => {
            return HttpResponse::InternalServerError().finish();
        }
    };
    store.max_byte_size = body.max_byte_size;
    let defaults = StoreDefaults { max_byte_size: body.max_byte_size };
    if let Err(_error) = store.update_store_defaults(&defaults) {
        return HttpResponse::InternalServerError().finish();
    }
    config.max_byte_size = body.max_byte_size;
    if let Err(_error) = update_config(&mut config, &data.config_location) {
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok().finish()    
}
