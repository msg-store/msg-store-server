use crate::{
    api::{
        lower::{
            lock,
            error_codes::{self, log_err},
            file_storage::get_buffer,
            Database,
            file_storage::{
                FileStorage
            },
            Either
        }
    },
    AppData,
};
use actix_web::{
    web::{
        Data, Bytes, 
    },
    Error
};
use futures::{
    stream::Stream,
    task::{Context, Poll}
};
use log::{
    error
};
use msg_store::{Uuid, Store};
use std::{
    fs::File,
    pin::Pin,
    process::exit,
    io::{BufReader, Read},
    sync::{Arc, Mutex}
};

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

pub fn handle(
    store: &Mutex<Store>,
    database_mutex: &Mutex<Database>,
    file_storage_option: &Option<Mutex<FileStorage>>,
    uuid_option: Option<Arc<Uuid>>,
    priority_option: Option<u32>,
    reverse_option: bool
) -> Result<Option<Either<ReturnBody, String>>, &'static str> {
    let uuid = {
        let store = lock(&store)?;
        match store.get(uuid_option, priority_option, reverse_option) {
            Ok(uuid) => match uuid {
                Some(uuid) => Ok(uuid),
                None => return Ok(None)
            },
            Err(error) => {
                log_err(error_codes::SYNC_ERROR, file!(), line!(), error.to_string());
                Err(error_codes::SYNC_ERROR)
            }
        }
    }?;
    let msg = {
        let mut database = lock(&database_mutex)?;
        match database.get(uuid.clone()) {
            Ok(msg) => Ok(msg),
            Err(error) => {
                log_err(error_codes::DATABASE_ERROR, file!(), line!(), error);
                Err(error_codes::DATABASE_ERROR)
            }
        }
    }?;
    let file_buffer = {
        if let Some(file_storage_mutex) = &file_storage_option {
            let file_storage = lock(file_storage_mutex)?;
            match get_buffer(&file_storage.path, &uuid) {
                Ok(buffer_option) => Ok(buffer_option),
                Err(error_code) => {
                    log_err(error_code, file!(), line!(), "");
                    Err(error_code)
                }
            }?
        } else {
            None
        }
    };
    if let Some((file_buffer, file_size)) = file_buffer {
        let msg_header = match String::from_utf8(msg.to_vec()) {
            Ok(msg_header) => Ok(msg_header),
            Err(error) => {
                log_err(error_codes::COULD_NOT_PARSE_CHUNK, file!(), line!(), error.to_string());
                Err(error_codes::COULD_NOT_PARSE_CHUNK)
            }
        }?;
        let body = ReturnBody::new(format!("uuid={}&{}?", uuid.to_string(), msg_header), file_size, file_buffer);
        Ok(Some(Either::A(body)))
    } else {
        let msg = match String::from_utf8(msg.to_vec()) {
            Ok(msg) => Ok(msg),
            Err(error) => {
                log_err(error_codes::COULD_NOT_PARSE_CHUNK, file!(), line!(), error.to_string());
                Err(error_codes::COULD_NOT_PARSE_CHUNK)
            }
        }?;
        Ok(Some(Either::B(format!("uuid={}?{}", uuid.to_string(), msg))))
    }
}
