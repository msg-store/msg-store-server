use bytes::Bytes;
use crate::AppData;
use crate::api::lower::error_codes;
use crate::api::lower::msg::add::{handle, Chunky};
use actix_web::web::{ Data, Payload };
use actix_web::{HttpResponse};
use futures::{Stream, StreamExt};
use log::info;
use serde::{Deserialize, Serialize};
use std::process::exit;
use std::task::Poll;

pub struct PayloadBridge(Payload);

impl Stream for PayloadBridge {
    type Item = Result<Bytes, &'static str>;
    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Option<Self::Item>> {
        let chunk_option = match self.0.poll_next_unpin(cx) {
            Poll::Ready(chunk_option) => chunk_option,
            Poll::Pending => return Poll::Pending
        };
        let chunk_result = match chunk_option {
            Some(chunk_result) => chunk_result,
            None => return Poll::Ready(None)
        };
        match chunk_result {
            Ok(chunk) => Poll::Ready(Some(Ok(Bytes::copy_from_slice(&chunk)))),
            Err(error) => {
                error_codes::log_err(error_codes::PAYLOAD_ERROR, file!(), line!(), error.to_string());
                Poll::Ready(Some(Err(error_codes::PAYLOAD_ERROR)))
            }
        }
    }
}

impl Chunky for PayloadBridge { }

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReturnBody {
    uuid: String,
}

const STATUS_400: &'static [&'static str] = &[
    error_codes::COULD_NOT_GET_CHUNK_FROM_PAYLOAD,
    error_codes::COULD_NOT_PARSE_CHUNK,
    error_codes::MISSING_HEADERS,
    error_codes::MALFORMED_HEADERS,
    error_codes::INVALID_PRIORITY,
    error_codes::MISSING_PRIORITY,
    error_codes::INVALID_BYTESIZE_OVERRIDE,
    error_codes::MISSING_BYTESIZE_OVERRIDE
];
const STATUS_403: &'static [&'static str] = &[
    error_codes::FILE_STORAGE_NOT_CONFIGURED
];
const STATUS_409: &'static [&'static str] = &[
    error_codes::MSG_EXCEEDES_STORE_MAX,
    error_codes::MSG_EXCEEDES_GROUP_MAX,
    error_codes::MSG_LACKS_PRIORITY
];

const ROUTE: &'static str = "POST /api/msg";
pub async fn http_handle(data: Data<AppData>, body: Payload) -> HttpResponse {
    info!("{}", ROUTE);
    match handle(&data.store, &data.file_storage, &data.stats, &data.db, &mut PayloadBridge(body)).await {
        Ok(uuid) => HttpResponse::Ok().json(ReturnBody { uuid: uuid.to_string() }),
        Err(error_code) => {
            if STATUS_400.contains(&error_code) {
                info!("{} 400 {}", ROUTE, error_code);
                HttpResponse::BadRequest().body(error_code)
            } else if STATUS_403.contains(&error_code) {
                info!("{} 403 {}", ROUTE, error_code);
                HttpResponse::Forbidden().body(error_code)
            } else if STATUS_409.contains(&error_code) {
                info!("{} 409 {}", ROUTE, error_code);
                HttpResponse::Conflict().body(error_code)
            } else {
                error_codes::log_err(error_code, file!(), line!(), "");
                exit(1)
            }
        }
    }
}
