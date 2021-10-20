#[macro_export]
macro_rules! fmt_result {
    ($result:expr) => {
        match $result {
            Ok(result) => Ok(result),
            Err(error) => Err(format!("{}::{} {}", file!(), line!(), error))
        }
    };
}

#[macro_export]
macro_rules! fmt_option {
    ($result:expr) => {
        match result {
            Some(result) => Ok(result),
            None => Err(format!("{}::{} Not found.", file!(), line!()))
        }
    };
}

#[macro_export]
macro_rules! fmt_err_msg {
    ($msg:expr) => {
        format!("{}::{} {}", file!(), line!(), $msg)
    };
}