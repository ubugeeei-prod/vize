use crate::{Result, jsonrpc};
use corsa::fast::{CompactString, FastMap};
use corsa::jsonrpc::{RawMessage, RequestId, RpcResponseError};
use corsa::lsp::{VirtualChange, VirtualDocument};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
};
use serde_json::{Value, json};
use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

pub fn run(args: &[String]) -> Result<()> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = BufWriter::new(stdout.lock());
    let mut last_configuration = Value::Null;
    let mut documents = FastMap::<CompactString, VirtualDocument>::default();
    let strict_shutdown_state = args.iter().find_map(|arg| {
        arg.strip_prefix("--strict-lsp-shutdown=")
            .map(PathBuf::from)
    });
    let ignore_shutdown = args.iter().any(|arg| arg == "--ignore-lsp-shutdown");
    let mut shutdown_received = false;
    loop {
        let Some(message) = jsonrpc::read_message(&mut reader)? else {
            if let Some(path) = strict_shutdown_state.as_deref() {
                append_shutdown_state(path, "context canceled")?;
            }
            return Ok(());
        };
        let method = message.method.unwrap_or_default();
        let has_params = message.params.is_some();
        let params = message.params.unwrap_or(Value::Null);
        match (message.id, method.as_str()) {
            (Some(id), "initialize") => {
                jsonrpc::write_message(
                    &mut writer,
                    &RawMessage::response(
                        id,
                        json!({
                            "capabilities": { "textDocumentSync": 1 },
                            "serverInfo": { "name": "mock-corsa" }
                        }),
                    ),
                )?;
                jsonrpc::write_message(
                    &mut writer,
                    &RawMessage::notification(
                        "window/logMessage",
                        json!({
                            "type": 3,
                            "message": "mock initialized"
                        }),
                    ),
                )?;
            }
            (Some(id), "custom/initializeAPISession") => {
                jsonrpc::write_message(
                    &mut writer,
                    &RawMessage::response(
                        id,
                        json!({
                            "sessionId": "session-1",
                            "pipe": "/tmp/mock-corsa.sock"
                        }),
                    ),
                )?;
            }
            (Some(id), "custom/lastConfiguration") => {
                jsonrpc::write_message(
                    &mut writer,
                    &RawMessage::response(id, last_configuration.clone()),
                )?;
            }
            (Some(id), "custom/overlayState") => {
                jsonrpc::write_message(
                    &mut writer,
                    &RawMessage::response(
                        id,
                        json!({
                            "documents": documents.values().cloned().collect::<Vec<_>>()
                        }),
                    ),
                )?;
            }
            (Some(id), "shutdown") => {
                if let Some(path) = strict_shutdown_state.as_deref() {
                    if has_params {
                        append_shutdown_state(path, "invalid params")?;
                        jsonrpc::write_message(
                            &mut writer,
                            &RawMessage::error(
                                id,
                                RpcResponseError {
                                    code: -32602,
                                    message: "expected no params, got null".into(),
                                    data: None,
                                },
                            ),
                        )?;
                        continue;
                    }
                    append_shutdown_state(path, "shutdown")?;
                }
                shutdown_received = true;
                if !ignore_shutdown {
                    jsonrpc::write_message(&mut writer, &RawMessage::response(id, Value::Null))?;
                }
            }
            (Some(id), _) => {
                jsonrpc::write_message(&mut writer, &RawMessage::response(id, Value::Null))?;
            }
            (None, "initialized") => {
                last_configuration = jsonrpc::send_request(
                    &mut reader,
                    &mut writer,
                    RequestId::integer(99),
                    "workspace/configuration",
                    json!({ "items": [{ "section": "typescript" }] }),
                )?;
            }
            (None, "textDocument/didOpen") => {
                let params: DidOpenTextDocumentParams = serde_json::from_value(params)?;
                let document = VirtualDocument::from_item(params.text_document);
                documents.insert(document.key(), document);
            }
            (None, "textDocument/didChange") => {
                let params: DidChangeTextDocumentParams = serde_json::from_value(params)?;
                if let Some(document) = documents.get_mut(params.text_document.uri.as_str()) {
                    let changes = params
                        .content_changes
                        .into_iter()
                        .map(VirtualChange::from)
                        .collect::<Vec<_>>();
                    document.apply_changes(&changes)?;
                }
            }
            (None, "textDocument/didClose") => {
                let params: DidCloseTextDocumentParams = serde_json::from_value(params)?;
                documents.remove(params.text_document.uri.as_str());
            }
            (None, "exit") => {
                if let Some(path) = strict_shutdown_state.as_deref() {
                    if has_params {
                        append_shutdown_state(path, "invalid exit params")?;
                        return Err("expected exit without params".into());
                    }
                    if !shutdown_received {
                        append_shutdown_state(path, "exit before shutdown")?;
                        return Err("expected shutdown before exit".into());
                    }
                    append_shutdown_state(path, "exit")?;
                }
                return Ok(());
            }
            (None, _) => {
                let _ = params;
            }
        }
    }
}

fn append_shutdown_state(path: &Path, state: &str) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{state}")?;
    Ok(())
}
