use crate::{
    api::{
        // from_value_prop, get_optional_priority, get_optional_uuid, http_reply,
        validate_uuid_string,
        // ws::{command::MSG_GET, Websocket},
        lock_or_exit,
        // ws_reply_with, Reply, http_bad_request, 
        // http_route_hit_log,
        lower::file_storage::get_buffer
    },
    AppData,
};
use actix_web::{
    web::{
        Data, Query, Bytes, 
        // BytesMut
    },
    HttpResponse,
    Error
};
// use actix_web_actors::ws::WebsocketContext;
use futures::{
    stream::Stream,
    task::{Context, Poll}
};
use log::{
    error, 
    // debug
};
use msg_store::Uuid;
use serde::{Deserialize, Serialize};
// use serde_json::Value;
use std::{
    fs::File,
    pin::Pin,
    process::exit,
    io::{BufReader, Read},
    sync::Arc
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MsgData {
    uuid: String,
    msg: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Info {
    uuid: Option<String>,
    priority: Option<u32>,
    reverse: Option<bool>,
}

pub struct InnerInfo {
    uuid: Option<Arc<Uuid>>,
    priority: Option<u32>,
    reverse: Option<bool>,
}

pub struct ReturnBody {
    header: String,
    msg: BufReader<File>,
    file_size: u64,
    bytes_read: u64,
    headers_sent: bool,
    msg_sent: bool
}
impl ReturnBody {
    pub fn new(header: String, file_size: u64, msg: BufReader<File>) -> ReturnBody {
        ReturnBody {
            header,
            file_size,
            bytes_read: 0,
            msg,
            headers_sent: false,
            msg_sent: false
        }
    }
}
impl Stream for ReturnBody {
    type Item = Result<Bytes, Error>;
    fn poll_next(
        mut self: Pin<&mut Self>, 
        _cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>> {
        // debug!("poll called");
        if self.msg_sent {
            return Poll::Ready(None);
        }
        if self.headers_sent {            
            let limit = self.file_size - self.bytes_read;
            if limit >= 665600 {
                let mut buffer = [0; 665600];
                let _bytes_read = match self.msg.read(&mut buffer) {
                    Ok(bytes_read) => bytes_read,
                    Err(error) => {
                        error!("ERROR_CODE: 980e2389-d8d4-448a-b60e-cb007f755d0b. Could not read to buffer: {}", error.to_string());
                        exit(1)
                    }
                };
                {
                    let mut body = self.as_mut().get_mut();
                    body.bytes_read += 665600;
                }
                return Poll::Ready(Some(Ok(Bytes::copy_from_slice(&buffer))));
            } else if limit == 0 {
                return Poll::Ready(None);
            } else {
                let mut buffer = Vec::with_capacity(limit as usize);
                if let Err(error) = self.msg.read_to_end(&mut buffer) {
                    error!("ERROR_CODE: e2865655-31f6-486f-8b8b-a360a506eb76. Could not read to buffer: {}", error.to_string());
                    exit(1)
                };
                {
                    let mut body = self.as_mut().get_mut();
                    body.msg_sent = true;
                }
                return Poll::Ready(Some(Ok(Bytes::copy_from_slice(&buffer))));
            }
        } else {
            {
                let mut body = self.as_mut().get_mut();
                body.headers_sent = true;
            }
            Poll::Ready(Some(Ok(Bytes::copy_from_slice(&self.header.as_bytes()))))
        }
    }
}

pub fn make_inner_info(info: Info) -> Result<InnerInfo, String> {
    let uuid = if let Some(uuid_string) = info.uuid {
        Some(validate_uuid_string(uuid_string)?)
    } else {
        None
    };
    let priority = info.priority;
    let reverse = info.reverse;
    Ok(InnerInfo {
        uuid,
        priority,
        reverse,
    })
}

pub fn handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    let info = match make_inner_info(info.into_inner()) {
        Ok(info) => info,
        Err(_message) => {
            return HttpResponse::BadRequest().finish();
        }
    };
    let reverse_option = if let Some(reverse) = info.reverse {
        reverse
    } else {
        false
    };
    let uuid = {
        let store = lock_or_exit(&data.store);
        match store.get(info.uuid, info.priority, reverse_option) {
            Ok(uuid) => match uuid {
                Some(uuid) => uuid,
                None => { return HttpResponse::Ok().finish() }
            },
            Err(error) => {
                error!("ERROR_CODE: 10ecc784-6e11-43ed-b631-e3d430f3e9af. Could not get msg from store: {}", error);
                exit(1);
            }
        }
    };

    let msg = {
        let mut db = lock_or_exit(&data.db);
        match db.get(uuid.clone()) {
            Ok(msg) => msg,
            Err(error) => {
                error!("ERROR_CODE: d6dce67e-c344-492c-b3a6-8c51ae9c8eb8. Could not get msg from database: {}", error);
                exit(1);
            }
        }
    };
    let file_buffer = {
        if let Some(file_storage_mutex) = &data.file_storage {
            let file_storage = lock_or_exit(file_storage_mutex);
            match get_buffer(&file_storage.path, &uuid) {
                Ok(buffer_option) => buffer_option,
                Err(error_code) => {
                    error!("ERROR_CODE: {}.", error_code);
                    exit(1);
                }
            }
        } else {
            None
        }
    };
    if let Some((file_buffer, file_size)) = file_buffer {
        let msg_header = match String::from_utf8(msg.to_vec()) {
            Ok(msg_header) => msg_header,
            Err(error) => {
                error!("ERROR_CODE: dca57b90-f567-4530-891e-932178ffd829. Could not get header from bytes: {}", error);
                exit(1);
            }
        };
        let body = ReturnBody::new(format!("uuid={}&{}?", uuid.to_string(), msg_header), file_size, file_buffer);
        HttpResponse::Ok().streaming(body)
    } else {
        let msg = match String::from_utf8(msg.to_vec()) {
            Ok(msg) => msg,
            Err(error) => {
                error!("ERROR_CODE: 17fd1f1a-2567-47a1-835f-a2277b7957b7. Could not get msg from bytes: {}", error);
                exit(1);
            }
        };
        HttpResponse::Ok().body(format!("uuid={}?{}", uuid.to_string(), msg))
    }
    
    
    // let is_file = {
    //     if let Some(file_list) = &data.file_list {
    //         let file_list = lock_or_exit(&file_list);
    //         if file_list.contains(&uuid) {
    //             true            
    //         } else {
    //             false
    //         }
    //     } else {
    //         false
    //     }
    // };
    // if is_file {
    //     if let None = data.file_list {

    //     }
    //     let mut db = lock_or_exit(&data.db);
    //     let msg_header = match db.get(uuid) {
    //         Ok(msg) => msg,
    //         Err(error) => {
    //             error!("ERROR_CODE: d6dce67e-c344-492c-b3a6-8c51ae9c8eb8. Could not get msg from database: {}", error);
    //             exit(1);
    //         }
    //     };
    //     let file_path = {
    //         if let Some(file_storage_path) = &data.file_storage_path {
    //             let mut file_path = file_storage_path.to_path_buf();
    //             file_path.push(uuid.to_string());
    //             file_path
    //         } else {
    //             error!("ERROR_CODE: 049a3118-2143-451e-81a4-eea8249231ad. Could not find file storage path.");
    //             exit(1);
    //         }
    //     };
    //     let file = match File::open(file_path) {
    //         Ok(file) => file,
    //         Err(error) => {
    //             error!("ERROR_CODE: 934484be-57f1-4d9e-9734-94210c4d18dd. Could not open file: {}", error);
    //             exit(1)
    //         }
    //     };
    //     let metadata = match file.metadata() {
    //         Ok(metadata) => metadata,
    //         Err(error) => {
    //             error!("ERROR_CODE: ed6dcc3d-ed0d-4e0d-8955-85c7050a373b. Could not get file metadata: {}", error);
    //             exit(1)
    //         }
    //     };
    //     let file_size = metadata.len();
    //     let file_buffer = BufReader::new(file);
    //     let msg_header = match String::from_utf8(msg_header.to_vec()) {
    //         Ok(msg_header) => msg_header,
    //         Err(error) => {
    //             error!("ERROR_CODE: dca57b90-f567-4530-891e-932178ffd829. Could not get header from bytes: {}", error);
    //             exit(1);
    //         }
    //     };
    //     let body = ReturnBody::new(format!("uuid={}&{}?", uuid.to_string(), msg_header), file_size, file_buffer);
    //     HttpResponse::Ok().streaming(body)
    // } else {
    //     let mut db = lock_or_exit(&data.db);
    //     let msg = match db.get(uuid) {
    //         Ok(msg) => msg,
    //         Err(error) => {
    //             error!("ERROR_CODE: d6dce67e-c344-492c-b3a6-8c51ae9c8eb8. Could not get msg from database: {}", error);
    //             exit(1);
    //         }
    //     };
    //     let msg = match String::from_utf8(msg.to_vec()) {
    //         Ok(msg) => msg,
    //         Err(error) => {
    //             error!("ERROR_CODE: 17fd1f1a-2567-47a1-835f-a2277b7957b7. Could not get msg from bytes: {}", error);
    //             exit(1);
    //         }
    //     };
    //     HttpResponse::Ok().body(format!("uuid={}?{}", uuid.to_string(), msg))
    // }
}

// const ROUTE: &'static str = "GET /api/msg";
// pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
//     http_route_hit_log(ROUTE, Some(info.clone()));
//     let info = match make_inner_info(info.into_inner()) {
//         Ok(info) => info,
//         Err(message) => {
//             return http_bad_request(ROUTE, message);
//         }
//     };
//     http_reply(ROUTE, handle(data, info))
// }

// pub fn ws_handle(ctx: &mut WebsocketContext<Websocket>,data: Data<AppData>, info: Value) {
//     http_route_hit_log(MSG_GET, Some(info.clone()));
//     let mut reply = ws_reply_with(ctx, MSG_GET);
//     let priority = match get_optional_priority(&info) {
//         Ok(priority) => priority,
//         Err(message) => {
//             return reply(Reply::BadRequest(message));
//         }
//     };
//     let uuid = match get_optional_uuid(&info) {
//         Ok(uuid) => uuid,
//         Err(message) => return reply(Reply::BadRequest(message)),
//     };
//     let reverse = match from_value_prop::<bool, _>(&info, "reverse", "boolean") {
//         Ok(reverse) => reverse,
//         Err(message) => return reply(Reply::BadRequest(message)),
//     };
//     reply(handle(
//         data,
//         InnerInfo {
//             priority,
//             uuid,
//             reverse,
//         },
//     ));
// }
