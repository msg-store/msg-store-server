use crate::{
    api::{
        http_route_hit_log,
        lower::msg::add::handle
    },
    AppData,
};
use actix_web::{
    web::{ Data,Payload },
    HttpResponse,
};
// use log::error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Body {
    priority: u32,
    msg: String,
}

// NEW BODY => PRIORITY=1 MSG=my really long msg ...

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReturnBody {
    uuid: String,
}

const ROUTE: &'static str = "POST /api/msg";
pub async fn http_handle(data: Data<AppData>, body: Payload) -> HttpResponse {
    http_route_hit_log::<()>(ROUTE, None);
    match handle(&data.store, &data.file_storage, &data.stats, &data.db, &mut body.into_inner()).await {
        Ok(uuid) => HttpResponse::Ok().json(ReturnBody { uuid: uuid.to_string() }),
        Err(_error_code) => {
            // TODO: Error handling
            HttpResponse::InternalServerError().finish()
        }
    }
    // http_reply(ROUTE, handle(data, body).await)
}
