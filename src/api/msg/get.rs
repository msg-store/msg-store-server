use crate::{
    api::{
        http_route_hit_log,
        lower::{
            error_codes,
            msg::get::handle,
            Either
        }
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
use futures::{
    stream::Stream,
    task::{Context, Poll}
};
use log::{
    error,
    info
};
use msg_store::Uuid;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    pin::Pin,
    process::exit,
    io::{BufReader, Read}
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

const ROUTE: &'static str = "GET /api/msg";
pub fn http_handle(data: Data<AppData>, info: Query<Info>) -> HttpResponse {
    http_route_hit_log(ROUTE, Some(info.clone()));
    let uuid = if let Some(uuid_string) = &info.uuid {
        match Uuid::from_string(&uuid_string) {
            Ok(uuid) => Some(uuid),
            Err(_) => {
                info!("{} 400 {}", ROUTE, error_codes::INVALID_UUID);
                return HttpResponse::BadRequest().body(error_codes::INVALID_UUID);
            } 
        }
    } else {
        None
    };
    let priority = info.priority;
    let reverse = if let Some(reverse) = info.reverse {
        reverse
    } else {
        false
    };
    let msg_option = match handle(
        &data.store, 
        &data.db, 
        &data.file_storage, 
        uuid, 
        priority, 
        reverse) {
        Ok(message_option) => message_option,
        Err(error_code) => {
            error_codes::log_err(error_code, file!(), line!(), "");
            exit(1);
        }
    };
    let msg_type = match msg_option {
        Some(msg_type) => msg_type,
        None => {
            info!("{} 200 No Message", ROUTE);
            return HttpResponse::Ok().finish()
        }
    };
    let buffer = match msg_type {
        Either::A(buffer) => buffer,
        Either::B(msg) => {
            info!("{} 200 {}", ROUTE, msg);
            return HttpResponse::Ok().body(msg)
        }
    };
    info!("{} 200 {}", ROUTE, buffer.header);
    HttpResponse::Ok().streaming(buffer)
}
