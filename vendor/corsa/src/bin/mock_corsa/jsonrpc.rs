use crate::Result;
use corsa::jsonrpc::{RawMessage, RequestId, read_frame, write_frame};
use serde_json::Value;
use std::io::{BufRead, Write};

pub fn read_message<R: BufRead>(reader: &mut R) -> Result<Option<RawMessage>> {
    match read_frame(reader) {
        Ok(payload) => Ok(Some(serde_json::from_slice(&payload)?)),
        Err(corsa::CorsaError::Closed(_)) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub fn write_message<W: Write>(writer: &mut W, message: &RawMessage) -> Result<()> {
    let body = serde_json::to_vec(message)?;
    write_frame(writer, &body)?;
    Ok(())
}

pub fn send_request<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    id: RequestId,
    method: &str,
    params: Value,
) -> Result<Value> {
    write_message(writer, &RawMessage::request(id.clone(), method, params))?;
    loop {
        let Some(message) = read_message(reader)? else {
            return Err("unexpected eof".into());
        };
        match (&message.id, &message.method) {
            (Some(inbound), None) if inbound == &id => {
                if let Some(error) = message.error {
                    return Err(format!("client error {}: {}", error.code, error.message).into());
                }
                return Ok(message.result.unwrap_or(Value::Null));
            }
            _ => {}
        }
    }
}
