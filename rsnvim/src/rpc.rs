use rmpv::{encode, Value};
use std::io::Write;

use crate::error::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum RpcMessage {
    RpcRequest {
        msgid: u64,
        method: String,
        params: Vec<Value>,
    }, // 0
    RpcResponse {
        msgid: u64,
        error: Value,
        result: Value,
    }, // 1
    RpcNotification {
        method: String,
        params: Vec<Value>,
    }, // 2
}

macro_rules! args_as_value {
    ($($arg:expr), *) => {{
        let mut vec = Vec::new();
        $(
            vec.push(Value::from($arg));
        )*
        Value::from(vec)
    }}
}

/// Creates a vector of rmpv::Value from the input args
#[macro_export]
macro_rules! value_vec {
    ($($arg:expr), *) => {{
        let mut vec = Vec::new();
        $(
            vec.push(Value::from($arg));
        )*
        vec
    }}
}

/// Encode as a MessagePack RPC message
pub fn encode<W: Write>(writer: &mut W, msg: RpcMessage) -> Result<(), Error> {
    match msg {
        RpcMessage::RpcRequest {
            msgid,
            method,
            params,
        } => {
            let val = args_as_value!(0, msgid, method, params);
            encode::write_value(writer, &val)?;
        }
        RpcMessage::RpcResponse {
            msgid,
            error,
            result,
        } => {
            let val = args_as_value!(1, msgid, error, result);
            encode::write_value(writer, &val)?;
        }
        RpcMessage::RpcNotification { method, params } => {
            let val = args_as_value!(2, method, params);
            encode::write_value(writer, &val)?;
        }
    };

    writer.flush()?;

    Ok(())
}
