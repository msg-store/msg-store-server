use crate::{
    api::{
        // get_require_msg, get_required_priority, 
        http_reply,
        // ws::{command::MSG_POST, Websocket},
        // ws_reply_with, 
        Reply, lock_or_exit, 
        http_route_hit_log, FileManager,
        lower::msg::add::handle
    },
    AppData,
};
use actix_web::{
    dev::Payload as DevPayload,
    web::{
        Data, 
        // Json, 
        BytesMut,
        Payload
    },
    HttpResponse,
};

use futures::StreamExt;
use log::{
    error, 
    // debug
};
use msg_store::errors::Error;
use msg_store_db_plugin::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::BTreeMap,
    process::exit
};

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
        Err(error_code) => {
            HttpResponse::InternalServerError().finish()
        }
    }
    // http_reply(ROUTE, handle(data, body).await)
}

// pub fn ws_handle(ctx: &mut ws::WebsocketContext<Websocket>, data: Data<AppData>, obj: Value) {
//     http_route_hit_log(MSG_POST, Some(obj.clone()));
//     ws_reply_with(ctx, MSG_POST)(handle(data, obj));
// }

// pub fn handle(data: Data<AppData>, body: Value) -> Reply<ReturnBody> {

//     let msg = match get_require_msg(&body) {
//         Ok(msg) => msg,
//         Err(message) => return Reply::BadRequest(message),
//     };
//     let msg_byte_size = msg.len() as u32;
//     let add_result = {
//         let mut store = lock_or_exit(&data.store);
//         let priority = match get_required_priority(&body) {
//             Ok(priority) => priority,
//             Err(message) => return Reply::BadRequest(message),
//         };        
//         match store.add(priority, msg_byte_size) {
//             Ok(add_result) => add_result,
//             Err(error) => match error {
//                 Error::ExceedesStoreMax => {
//                     return Reply::Conflict(
//                         "Message byte size exceeds the max byte size limit allowed by the store"
//                             .to_string(),
//                     );
//                 }
//                 Error::ExceedesGroupMax => {
//                     return Reply::Conflict(
//                         "Message byte size exceeds the max byte size limit allowed by the group"
//                             .to_string(),
//                     );
//                 }
//                 Error::LacksPriority => {
//                     return Reply::Conflict(
//                         "The store has reached max capcity and could not accept message".to_string(),
//                     );
//                 }
//                 error => {
//                     error!("ERROR_CODE: c911f827-35ec-42aa-8ca3-2b10b68209c9. {}", error.to_string());
//                     exit(1)
//                 },
//             },
//         }
//     };    
    
//     // remove msgs from db
//     let mut deleted_count = 0;
//     for uuid in add_result.msgs_removed.into_iter() {
//         let mut db = lock_or_exit(&data.db);
//         if let Err(error) = db.del(uuid) {
//             error!("ERROR_CODE: 0753a0a2-5436-44e1-bb05-6e81193ad9e7. Could not remove msg from database: {}", error);
//             exit(1);
//         }
//         deleted_count += 1;
//     }
//     {
//         let mut stats = lock_or_exit(&data.stats);
//         stats.deleted += deleted_count;
//         stats.inserted += 1;
//     }
//     {
//         let mut db = lock_or_exit(&data.db);
//         if let Err(error) = db.add(add_result.uuid, msg, msg_byte_size) {
//             error!("ERROR_CODE: f106eed6-1c47-4437-b9c3-082a4c5393af. Could not add msg to database: {}", error);
//             exit(1);
//         }
//     }
//     Reply::OkWData(ReturnBody {
//         uuid: add_result.uuid.to_string(),
//     })
// }

// const ROUTE: &'static str = "POST /api/msg";
// pub fn http_handle(data: Data<AppData>, body: Json<Value>) -> HttpResponse {
//     http_route_hit_log(ROUTE, Some(body.clone()));
//     http_reply(ROUTE, handle(data, body.into_inner()))
// }

// pub fn ws_handle(ctx: &mut ws::WebsocketContext<Websocket>, data: Data<AppData>, obj: Value) {
//     http_route_hit_log(MSG_POST, Some(obj.clone()));
//     ws_reply_with(ctx, MSG_POST)(handle(data, obj));
// }