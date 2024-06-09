use rmpv::{decode, encode, Value};
use std::io::{Read, Write};

use crate::error::Error;

/// The possible RPC messages sent to/from the Neovim
/// instance.
#[derive(Debug, PartialEq, Clone)]
pub enum RpcMessage {
    /// RPC request for this server to respond to.
    ///
    /// An RPC request is assigned 0u64 by Neovim.
    RpcRequest {
        msgid: u64,
        method: String,
        params: Vec<Value>,
    },
    /// Response to an RPC call made from this server
    ///
    /// An RPC response is assigned 1u64 by Neovim.
    RpcResponse {
        msgid: u64,
        error: Value,
        result: Value,
    },
    /// RPC notification from the Neovim instance
    ///
    /// An RPC notification is assigned 2u64 by Neovim.
    RpcNotification { method: String, params: Vec<Value> },
}

/// Iterate through Rust types, converting them to a rmpv::Value,
/// concatenating them into a Vec and converting the Vec to a
/// rmpv::Value::Array.
macro_rules! args_as_value {
    ($($arg:expr), *) => {{
        Value::from(vec![$(Value::from($arg), )*])
    }}
}

/// Iterate through Rust types, converting them to a rmpv::Value,
/// concatenating them into a Vec<rmpv::Value>.
#[macro_export]
macro_rules! value_vec {
    ($($arg:expr), *) => {{
        vec![$(AsValue::convert(&$arg), )*]
    }}
}

/// Returns a Vec<rmpv::Value> if the input is a rmpv::Value::Array,
/// otherwise return an error.
macro_rules! try_arr {
    ($exp:expr) => {
        match $exp {
            Value::Array(_) => $exp.as_array().unwrap(),
            _ => {
                return Err(Error::DecodingError(
                    "RPC element is not an array".to_string(),
                ))
            }
        }
    };
}

/// Returns a &str if the input is a rmpv::Value::String,
/// otherwise return an error.
macro_rules! try_str {
    ($exp:expr) => {
        match $exp {
            Value::String(_) => $exp.as_str().unwrap(),
            _ => return Err(Error::DecodingError("RPC element not a string".to_string())),
        }
    };
}

/// Returns a u64 if the input is a rmpv::Value::Integer,
/// otherwise return an error.
macro_rules! try_int {
    ($exp:expr) => {
        match $exp {
            Value::Integer(_) => $exp.as_u64().unwrap(),
            _ => return Err(Error::DecodingError("RPC element not a string".to_string())),
        }
    };
}

/// Decode MessagePack RPC message
///
/// Returns an `RpcMessage` containing one of
/// - `RpcRequest`: RPC call for this server to respond to
/// - `RpcResponse`: response to an RPC call made from this server
/// - `RpcNotification`: RPC notification from the Neovim instance
pub fn decode<R: Read>(reader: &mut R) -> Result<RpcMessage, Error> {
    let arr = decode::read_value(reader).unwrap();
    match arr {
        Value::Array(_) => (),
        _ => {
            return Err(Error::DecodingError(
                "RPC message must be an array".to_string(),
            ))
        }
    }

    match arr[0].as_u64() {
        Some(0) => {
            let msgid = try_int!(&arr[1]);
            let method = try_str!(&arr[2]).to_string();
            let params = try_arr!(&arr[3]).to_vec();

            Ok(RpcMessage::RpcRequest {
                msgid,
                method,
                params,
            })
        }
        Some(1) => {
            let msgid = try_int!(&arr[1]);
            let error = arr[2].clone();
            let result = arr[3].clone();

            Ok(RpcMessage::RpcResponse {
                msgid,
                error,
                result,
            })
        }
        Some(2) => {
            let method = try_str!(&arr[1]).to_string();
            let params = try_arr!(&arr[2]).to_vec();

            Ok(RpcMessage::RpcNotification { method, params })
        }
        _ => Err(Error::DecodingError(
            "RPC message does not contain type".to_string(),
        )),
    }
}

/// Encode MessagePack RPC message and send to Neovim instance.
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
