use actix_web::{
    web::{Data, Query},
    HttpResponse,
};
use crate::AppData;
use log::info;
use msg_store::api::{
    group::rm,
    error_codes
};
use serde::{Deserialize, Serialize};
use std::process::exit;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Info {
    priority: u32,
}

const ROUTE: &'static str = "DEL /api/group";
pub fn handle_http(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    info!("{} priority: {}", ROUTE, info.priority);    
    if let Err(error_code) = rm::handle(
        &data.store, 
        &data.db, 
        &data.file_storage, 
        &data.stats, info.priority) {
            error_codes::log_err(error_code, file!(), line!(), "");
            exit(1);
    }
    info!("{} 200", ROUTE);
    HttpResponse::Ok().finish()
}

// pub fn handle_ws(ctx: &mut WebsocketContext<Websocket>, data: Data<AppData>, info: Value) {
//     http_route_hit_log(GROUP_DELETE, Some(info.clone()));
//     let mut reply = ws_reply_with(ctx, GROUP_DELETE);
//     let priority = match from_value_prop_required(&info, "priority", "number") {
//         Ok(priority) => priority,
//         Err(message) => {
//             reply(Reply::BadRequest(message));
//             return;
//         }
//     };
//     reply(handle(data, Info { priority }));
// }
