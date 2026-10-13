use crate::Result;
use serde_json::{Value, json};
use std::{
    fs::{OpenOptions, create_dir_all},
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    time::Duration,
};

const MSG_REQUEST: u8 = 1;
const MSG_CALL_RESPONSE: u8 = 2;
const MSG_RESPONSE: u8 = 4;
const MAX_BIN_BYTES: usize = 512 * 1024 * 1024;

pub fn run(cwd: String, callbacks: Vec<String>) -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());
    loop {
        let (kind, method, payload) = read_tuple(&mut reader)?;
        if kind != MSG_REQUEST {
            return Err(format!("unexpected message type {kind}").into());
        }
        let method = String::from_utf8(method)?;
        record_method(method.as_str());
        let params: Value = serde_json::from_slice(&payload)?;
        record_params(method.as_str(), &params);
        let response = match method.as_str() {
            "initialize" => json!({
                "useCaseSensitiveFileNames": true,
                "currentDirectory": cwd,
            }),
            "describeCapabilities" => crate::common::capabilities(),
            "batchRequests" => batch_requests(params),
            "parseConfigFile" => parse_config(&mut reader, &mut writer, &callbacks, params)?,
            "updateSnapshot" => {
                crate::common::snapshot_from_update_params("/workspace/tsconfig.json", &params)
            }
            "getDiagnosticsForSnapshot" => {
                crate::common::snapshot_diagnostics(json!("/workspace/src/index.ts"))
            }
            "getDiagnosticsForProject" => {
                crate::common::project_diagnostics(json!("/workspace/src/index.ts"))
            }
            "getDiagnosticsForFile" => crate::common::file_diagnostics(
                params
                    .get("file")
                    .cloned()
                    .unwrap_or_else(|| json!("/workspace/src/index.ts")),
            ),
            "getHoverAtPosition" => crate::common::hover(),
            "getDefinitionAtPosition" => crate::common::definition(),
            "getReferencesAtPosition" => crate::common::references(),
            "getRenameAtPosition" => crate::common::rename(
                params
                    .get("newName")
                    .and_then(Value::as_str)
                    .unwrap_or("renamedValue"),
            ),
            "getCompletionAtPosition" => crate::common::completion(),
            "getSourceFile" => {
                write_tuple(&mut writer, MSG_RESPONSE, method.as_bytes(), b"source-file")?;
                continue;
            }
            "typeToTypeNode" => {
                write_tuple(&mut writer, MSG_RESPONSE, method.as_bytes(), b"type-node")?;
                continue;
            }
            "printNode" => json!("print:type-node"),
            "ping" => json!("pong"),
            "echo" => {
                write_tuple(&mut writer, MSG_RESPONSE, method.as_bytes(), &payload)?;
                continue;
            }
            _ => json!(null),
        };
        write_tuple(
            &mut writer,
            MSG_RESPONSE,
            method.as_bytes(),
            &serde_json::to_vec(&response)?,
        )?;
    }
}

fn batch_requests(params: Value) -> Value {
    let responses = params
        .get("requests")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|request| {
            let method = request
                .get("method")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let params = request.get("params").cloned().unwrap_or(Value::Null);
            let mut response = json!({ "method": method });
            if let Some(error) = crate::common::missing_project_message(&params) {
                response["error"] = Value::String(error);
                return response;
            }
            response["result"] = batch_request_result(method, &params).unwrap_or(Value::Null);
            response
        })
        .collect::<Vec<_>>();
    json!({ "responses": responses })
}

fn batch_request_result(method: &str, params: &Value) -> Option<Value> {
    match method {
        "getSymbolsAtPositions" | "getSymbolsAtLocations" => {
            Some(repeated_symbol_responses(batch_request_item_count(params)))
        }
        "getTypeOfSymbol"
        | "getDeclaredTypeOfSymbol"
        | "getTypeAtPosition"
        | "getTypeAtLocation"
        | "getConstraintOfType" => Some(crate::common::type_response("t0000000000000001")),
        "getTypeArguments" => Some(repeated_type_responses(1)),
        "getTypesOfSymbols" | "getTypeAtLocations" | "getTypesAtPositions" => {
            Some(repeated_type_responses(batch_request_item_count(params)))
        }
        "getSymbolOfType"
        | "getAliasedSymbol"
        | "getImmediateAliasedSymbol"
        | "getPropertyOfType" => Some(crate::common::symbol("value")),
        "getExportsOfModule" => Some(json!([crate::common::symbol("value")])),
        "isTypeAssignableTo" => Some(json!(true)),
        "typeToString" => Some(json!("type:string")),
        _ => None,
    }
}

fn batch_request_item_count(params: &Value) -> usize {
    params
        .as_object()
        .and_then(|value| {
            value
                .get("symbols")
                .or_else(|| value.get("locations"))
                .or_else(|| value.get("positions"))
                .and_then(Value::as_array)
        })
        .map(Vec::len)
        .unwrap_or(1)
}

fn repeated_type_responses(count: usize) -> Value {
    Value::Array(
        (0..count)
            .map(|_| crate::common::type_response("t0000000000000001"))
            .collect(),
    )
}

fn repeated_symbol_responses(count: usize) -> Value {
    Value::Array((0..count).map(|_| crate::common::symbol("value")).collect())
}

fn record_params(method: &str, params: &Value) {
    let Ok(dir) = std::env::var("CORSA_MOCK_PARAMS_DIR") else {
        return;
    };
    if create_dir_all(&dir).is_err() {
        return;
    }
    let path = Path::new(&dir).join(format!("{method}.jsonl"));
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", params);
    }
}

fn record_method(method: &str) {
    maybe_delay(method);
    let Ok(dir) = std::env::var("CORSA_MOCK_COUNT_DIR") else {
        return;
    };
    if create_dir_all(&dir).is_err() {
        return;
    }
    let path = Path::new(&dir).join(format!("{method}.count"));
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "1");
    }
}

fn maybe_delay(method: &str) {
    let Ok(delay_ms) = std::env::var("CORSA_MOCK_DELAY_MS") else {
        return;
    };
    if method != "initialize" && method != "describeCapabilities" {
        return;
    }
    if let Ok(delay_ms) = delay_ms.parse::<u64>() {
        std::thread::sleep(Duration::from_millis(delay_ms));
    }
}

fn parse_config<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    callbacks: &[String],
    params: Value,
) -> Result<Value> {
    let file = params
        .get("file")
        .and_then(Value::as_str)
        .unwrap_or("/workspace/tsconfig.json");
    if file.starts_with("/virtual/") && callbacks.iter().any(|name| name == "readFile") {
        write_tuple(
            writer,
            6,
            b"readFile",
            serde_json::to_string(file)?.as_bytes(),
        )?;
        let (kind, _, payload) = read_tuple(reader)?;
        if kind != MSG_CALL_RESPONSE {
            return Err("expected callback response".into());
        }
        let response: Value = serde_json::from_slice(&payload)?;
        return Ok(json!({
            "options": { "virtual": response.get("content").is_some() },
            "fileNames": ["/workspace/src/index.ts"],
        }));
    }
    Ok(json!({
        "options": { "strict": true },
        "fileNames": ["/workspace/src/index.ts"],
    }))
}

fn read_tuple<R: Read>(reader: &mut R) -> Result<(u8, Vec<u8>, Vec<u8>)> {
    let mut prefix = [0_u8; 2];
    reader.read_exact(&mut prefix)?;
    if prefix[0] != 0x93 {
        return Err("invalid tuple marker".into());
    }
    let method = read_bin(reader)?;
    let payload = read_bin(reader)?;
    Ok((prefix[1], method, payload))
}

fn read_bin<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut tag = [0_u8; 1];
    reader.read_exact(&mut tag)?;
    let len = match tag[0] {
        0xc4 => read_len(reader, 1)?,
        0xc5 => read_len(reader, 2)?,
        0xc6 => read_len(reader, 4)?,
        _ => return Err("invalid bin tag".into()),
    };
    if len > MAX_BIN_BYTES {
        return Err(format!(
            "msgpack bin is too large: {len} bytes exceeds {MAX_BIN_BYTES} byte safety limit"
        )
        .into());
    }
    let mut payload = Vec::new();
    payload.try_reserve_exact(len).map_err(|error| {
        format!("failed to reserve msgpack bin buffer for {len} bytes: {error}")
    })?;
    payload.resize(len, 0);
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

fn read_len<R: Read>(reader: &mut R, width: usize) -> Result<usize> {
    Ok(match width {
        1 => {
            let mut buf = [0_u8; 1];
            reader.read_exact(&mut buf)?;
            buf[0] as usize
        }
        2 => {
            let mut buf = [0_u8; 2];
            reader.read_exact(&mut buf)?;
            u16::from_be_bytes(buf) as usize
        }
        _ => {
            let mut buf = [0_u8; 4];
            reader.read_exact(&mut buf)?;
            u32::from_be_bytes(buf) as usize
        }
    })
}

fn write_tuple<W: Write>(writer: &mut W, kind: u8, method: &[u8], payload: &[u8]) -> Result<()> {
    writer.write_all(&[0x93, kind])?;
    write_bin(writer, method)?;
    write_bin(writer, payload)?;
    writer.flush()?;
    Ok(())
}

fn write_bin<W: Write>(writer: &mut W, payload: &[u8]) -> Result<()> {
    if payload.len() > MAX_BIN_BYTES {
        return Err(format!(
            "msgpack bin is too large: {} bytes exceeds {MAX_BIN_BYTES} byte safety limit",
            payload.len()
        )
        .into());
    }
    match payload.len() {
        0..=255 => writer.write_all(&[0xc4, payload.len() as u8])?,
        256..=65535 => {
            writer.write_all(&[0xc5])?;
            writer.write_all(&(payload.len() as u16).to_be_bytes())?;
        }
        _ => {
            writer.write_all(&[0xc6])?;
            writer.write_all(&(payload.len() as u32).to_be_bytes())?;
        }
    }
    writer.write_all(payload)?;
    Ok(())
}
