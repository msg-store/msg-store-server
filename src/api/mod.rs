pub mod export;
pub mod group;
pub mod group_defaults;
pub mod msg;
pub mod stats;
pub mod store;
pub mod ws;

use crate::config::StoreConfig;
use actix_web::HttpResponse;
use actix_web_actors::ws::WebsocketContext;
use msg_store::Uuid;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, json, to_string, Value};
use std::{fmt::Display, path::PathBuf, process::exit};

use self::ws::Websocket;

pub fn update_config(config: &StoreConfig, config_path: &Option<PathBuf>) -> Result<(), String> {
    let should_update = {
        let mut should_update = true;
        if let Some(no_update) = config.no_update {
            if no_update {
                should_update = false;
            }
        }
        should_update
    };
    if should_update {
        if let Some(config_path) = config_path {
            config.update_config_file(&config_path)?;
        }
    }
    Ok(())
}

pub enum Reply<T: Serialize> {
    Ok,
    OkWData(T),
    BadRequest(String),
    Conflict(String),
}

// pub enum OkReply<T: Serialize> {
//     Ok,
//     OkWData(T)
// }

// pub enum Error {
//     BadRequest(String),
//     Conflict(String)
// }

// pub fn if_ok<A, B, C, Z, Y>(ok_handle: Z, err_handle: Y) -> impl Fn(Result<A,B>) -> C
// where
//     Z: Fn(A) -> C + Copy,
//     Y: Fn(B) -> C + Copy
// {
//     move |r| {
//         match r {
//             Ok(a) => ok_handle(a),
//             Err(b) => err_handle(b)
//         }
//     }
// }

// pub fn mk_json() -> Value {
//     json!({  })
// }

// pub fn append_status<T: Serialize>(status: T) -> impl Fn(Value) -> Value {
//     move |mut value| {
//         value["status"] = json!(status);
//         value
//     }
// }

// pub fn append_cmd<T: Serialize>(cmd: T) -> impl Fn(Value) -> Value {
//     move |mut value| {
//         value["cmd"] = json!(cmd);
//         value
//     }
// }

// pub fn append_data<T: Serialize>(data: T) -> impl Fn(Value) -> Value {
//     move |mut value| {
//         value["data"] = json!(data);
//         value
//     }
// }

// pub fn append_error_message(message: String) -> impl Fn(Value) -> Value {
//     move |mut value| {
//         value["message"] = json!(message);
//         value
//     }
// }

// pub fn append_error_code(code: u32) -> impl Fn(Value) -> Value {
//     move |mut value| {
//         value["errorCode"] = json!(code);
//         value
//     }
// }

// pub fn convert_to_string() -> impl Fn(Value) -> String {
//     |value| {
//         match to_string(&value) {
//             Ok(value) => value,
//             Err(_error) => {
//                 exit(1);
//             }
//         }
//     }
// }

// pub fn get_prop<T1: DeserializeOwned,T2: Index + Copy>(prop: T2) -> impl Fn(Value) -> Result<Option<T1>, String> {
//     move |value| {
//         match value.get(prop) {
//             Some(property) => match from_value(property.to_owned()) {
//                 Ok(property) => Ok(Some(property)),
//                 Err(error) => Err(error.to_string())
//             },
//             None => Ok(None)
//         }
//     }
// }

// pub fn require_prop<T: DeserializeOwned>() -> impl Fn(Result<Option<T>, String>) -> Result<T, String> {
//     |result| {
//         match result {
//             Ok(option) => match option {
//                 Some(value) => Ok(value),
//                 None => Err("missing crap".to_string())
//             },
//             Err(error) => Err(error)
//         }
//     }
// }

// pub fn send_value<'a>(ctx: &'a mut WebsocketContext<Websocket>) -> impl FnMut(String) + 'a {
//     move |data| ctx.text(data)
// }

pub fn from_value_prop<'a, T: DeserializeOwned, T2: Into<String> + Display>(
    value: &Value,
    prop: &'static str,
    prop_type: T2,
) -> Result<Option<T>, String> {
    if let Some(value) = value.get(prop) {
        if let Ok(value) = from_value::<Option<T>>(value.clone()) {
            Ok(value)
        } else {
            Err(format!("/data/{} must be type {}", prop, prop_type))
        }
    } else {
        Ok(None)
    }
}

pub fn from_value_prop_required<'a, T: DeserializeOwned>(
    value: &Value,
    prop: &'static str,
    prop_type: &'static str,
) -> Result<T, String> {
    if let Some(value) = value.get(prop) {
        if let Ok(value) = from_value::<Option<T>>(value.clone()) {
            if let Some(value) = value {
                Ok(value)
            } else {
                Err(format!("/data/{} must be type {}", prop, prop_type))
            }
        } else {
            Err(format!("/data/{} must be type {}", prop, prop_type))
        }
    } else {
        Err(format!("/data/{} must be type {}", prop, prop_type))
    }
}

pub fn get_optional_number(value: &Value, prop: &'static str) -> Result<Option<u32>, String> {
    from_value_prop::<u32, _>(value, prop, "number")
}

// pub fn get_required_number(value: &Value, prop: &'static str) -> Result<u32, String> {
//     from_value_prop_required(value, prop, "number")
// }

// pub fn get_optional_string(value: &Value, prop: &'static str) -> Result<Option<String>, String> {
//     from_value_prop(value, prop, "string")
// }

pub fn get_required_string(value: &Value, prop: &'static str) -> Result<String, String> {
    from_value_prop_required(value, prop, "string")
}

pub fn get_required_priority(value: &Value) -> Result<u32, String> {
    from_value_prop_required(value, "priority", "number")
}

pub fn get_optional_priority(value: &Value) -> Result<Option<u32>, String> {
    from_value_prop::<u32, _>(value, "priority", "number")
}

pub fn get_optional_max_byte_size(value: &Value) -> Result<Option<u32>, String> {
    let mut max_byte_size = from_value_prop(value, "max_byte_size", "number")?;
    if let None = max_byte_size {
        max_byte_size = from_value_prop(value, "maxByteSize", "number")?;
    }
    Ok(max_byte_size)
}

pub fn validate_uuid_string(uuid_string: String) -> Result<Uuid, String> {
    match Uuid::from_string(&uuid_string) {
        Ok(uuid) => Ok(uuid),
        Err(_) => Err(format!("uuid string must be of <u128>-<u32>")),
    }
}

pub fn get_optional_uuid(value: &Value) -> Result<Option<Uuid>, String> {
    match from_value_prop::<String, _>(value, "uuid", "string") {
        Ok(uuid_string) => match uuid_string {
            Some(uuid_string) => match validate_uuid_string(uuid_string) {
                Ok(uuid) => Ok(Some(uuid)),
                Err(message) => Err(message),
            },
            None => Ok(None),
        },
        Err(message) => Err(message),
    }
}

pub fn get_required_uuid(value: &Value) -> Result<Uuid, String> {
    match from_value_prop_required::<String>(value, "uuid", "string") {
        Ok(uuid_string) => match validate_uuid_string(uuid_string) {
            Ok(uuid) => Ok(uuid),
            Err(message) => Err(message),
        },
        Err(message) => Err(message),
    }
}

pub fn get_require_msg(value: &Value) -> Result<String, String> {
    from_value_prop_required::<String>(value, "msg", "string")
}

pub fn ws_ok(cmd: &'static str) -> String {
    match to_string(&json!({ "cmd": cmd,  "status": 200 })) {
        Ok(msg) => msg,
        Err(_) => exit(1),
    }
}

pub fn ws_ok_w_data<T: Serialize>(cmd: &'static str, data: T) -> String {
    match to_string(&json!({ "cmd": cmd,  "status": 200, "data": data })) {
        Ok(msg) => msg,
        Err(_) => exit(1),
    }
}

pub fn ws_bad_request(cmd: &'static str, message: String) -> String {
    match to_string(&json!({ "cmd": cmd,  "status": 400, "message": message })) {
        Ok(msg) => msg,
        Err(_) => exit(1),
    }
}

pub fn ws_conflict(cmd: &'static str, message: String) -> String {
    match to_string(&json!({ "cmd": cmd,  "status": 409, "message": message })) {
        Ok(msg) => msg,
        Err(_) => exit(1),
    }
}

// pub fn ws_reply<T: Serialize>(reply: Reply<T>) -> Value {    
//     match reply {
//         Reply::Ok => append_status(200)(mk_json()),
//         Reply::OkWData(data) => compose!(append_status(200), append_data(data))(mk_json()),
//         Reply::BadRequest(message) => compose!(append_status(400), append_error_message(message))(mk_json()),
//         Reply::Conflict(message) => compose!(append_status(409), append_error_message(message))(mk_json())
//     }
// }

// pub fn ws_reply_with_2<'a>(ctx: &'a mut WebsocketContext<Websocket>) -> impl FnMut(String) + 'a {
//     move |message| {        
//         ctx.text(message)
//     }
// }

pub fn ws_reply_with<'a, T: Serialize>(
    ctx: &'a mut WebsocketContext<Websocket>,
    cmd: &'static str,
) -> impl FnMut(Reply<T>) + 'a {
    move |reply| {
        let message = match reply {
            Reply::Ok => ws_ok(cmd),
            Reply::OkWData(data) => ws_ok_w_data(cmd, data),
            Reply::BadRequest(message) => ws_bad_request(cmd, message),
            Reply::Conflict(message) => ws_conflict(cmd, message),
        };
        ctx.text(message)
    }
}

pub fn http_reply<T: Serialize>(reply: Reply<T>) -> HttpResponse {
    match reply {
        Reply::Ok => http_ok(),
        Reply::OkWData(data) => http_ok_w_data(data),
        Reply::BadRequest(message) => http_bad_request(message),
        Reply::Conflict(message) => http_conflict(message),
    }
}

pub fn http_ok() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn http_ok_w_data<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(data)
}

pub fn http_bad_request(message: String) -> HttpResponse {
    HttpResponse::BadRequest()
        .content_type("text/plain")
        .body(message)
}

pub fn http_conflict(message: String) -> HttpResponse {
    HttpResponse::Conflict()
        .content_type("text/plain")
        .body(message)
}

pub fn prepend_data_str(message: String) -> String {
    format!("/data/{}", message)
}

pub fn append_null(message: String) -> String {
    format!("{} | null", message)
}
