//! Initialization and optional host-capability negotiation.

use std::io;

use serde_json::{Value, json};
use vize_l0::cstr;

use super::super::InitializeParams;
use super::wire::{encode_error, encode_result};

pub(super) fn initialize(
    id: Value,
    params: Value,
    initialized: &mut bool,
    diagnostic_code_selectors: &mut bool,
) -> io::Result<Vec<u8>> {
    let params = match serde_json::from_value::<InitializeParams>(params) {
        Ok(params) => params,
        Err(error) => {
            return encode_error(&id, -32602, &cstr!("Invalid initialize params: {error}"));
        }
    };
    if !params
        .position_encodings
        .iter()
        .any(|encoding| encoding == "utf-8")
    {
        return encode_error(&id, -32602, "Vize requires the utf-8 position encoding");
    }
    *initialized = true;
    *diagnostic_code_selectors = params.capabilities.diagnostic_directive_code_selectors;
    encode_result(
        &id,
        json!({"positionEncoding": "utf-8", "diagnosticSource": "vize"}),
    )
}
