#![allow(dead_code)]

use serde_json::{json, Map, Value};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
};

#[path = "./e2e.rs"]
mod editor_e2e;

pub struct LspSession {
    child: Child,
    stdin: ChildStdin,
    buffer: Vec<u8>,
    next_id: i64,
    backlog: Vec<(String, Value)>,
}

impl LspSession {
    pub fn spawn(repo_root: &Path) -> Result<Self, String> {
        let server = match std::env::var("VIZE_LSP_BIN") {
            Ok(path) => std::path::PathBuf::from(path),
            Err(_) => editor_e2e::resolve_real_server_path(repo_root)?,
        };
        let mut child = Command::new(server)
            .arg("lsp")
            .current_dir(repo_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| format!("failed to spawn vize lsp: {error}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "vize lsp stdin is unavailable".to_string())?;
        Ok(Self {
            child,
            stdin,
            buffer: Vec::new(),
            next_id: 0,
            backlog: Vec::new(),
        })
    }

    pub fn initialize(&mut self, workspace_dir: &Path, options: Value) -> Result<Value, String> {
        let uri = file_url(workspace_dir)?;
        let result = self.request(
            "initialize",
            json!({
                "processId": std::process::id(),
                "rootUri": uri,
                "capabilities": {
                    "textDocument": {
                        "completion": {
                            "completionItem": {
                                "documentationFormat": ["markdown", "plaintext"]
                            }
                        }
                    }
                },
                "initializationOptions": options,
                "workspaceFolders": [{
                    "uri": uri,
                    "name": workspace_dir.file_name().and_then(|name| name.to_str()).unwrap_or("workspace")
                }]
            }),
        )?;
        self.notify("initialized", json!({}))?;
        Ok(result)
    }

    pub fn request(&mut self, method: &str, params: Value) -> Result<Value, String> {
        self.next_id += 1;
        let id = self.next_id;
        self.send(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }))?;
        loop {
            let message = self.read_message()?;
            if message.get("id").and_then(Value::as_i64) == Some(id) {
                if let Some(error) = message.get("error") {
                    return Err(format!("{method} failed: {error}"));
                }
                return Ok(message.get("result").cloned().unwrap_or(Value::Null));
            }
            if let Some(method) = message.get("method").and_then(Value::as_str) {
                self.backlog.push((
                    method.to_string(),
                    message.get("params").cloned().unwrap_or(Value::Null),
                ));
            }
        }
    }

    pub fn notify(&mut self, method: &str, params: Value) -> Result<(), String> {
        self.send(json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        }))
    }

    pub fn wait_for_notification<F>(&mut self, method: &str, predicate: F) -> Result<Value, String>
    where
        F: Fn(&Value) -> bool,
    {
        if let Some(index) = self
            .backlog
            .iter()
            .position(|(candidate, params)| candidate == method && predicate(params))
        {
            let (_, params) = self.backlog.remove(index);
            return Ok(params);
        }
        loop {
            let message = self.read_message()?;
            if let Some(candidate) = message.get("method").and_then(Value::as_str) {
                let params = message.get("params").cloned().unwrap_or(Value::Null);
                if candidate == method && predicate(&params) {
                    return Ok(params);
                }
                self.backlog.push((candidate.to_string(), params));
            }
        }
    }

    pub fn shutdown(mut self) -> Result<(), String> {
        let _ = self.request("shutdown", Value::Null);
        let _ = self.notify("exit", Value::Null);
        let _ = self.stdin.flush();
        let _ = self.child.kill();
        Ok(())
    }

    fn send(&mut self, message: Value) -> Result<(), String> {
        let body = serde_json::to_string(&message).map_err(|error| error.to_string())?;
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body)
            .map_err(|error| format!("failed to write LSP frame: {error}"))?;
        self.stdin
            .flush()
            .map_err(|error| format!("failed to flush LSP frame: {error}"))
    }

    fn read_message(&mut self) -> Result<Value, String> {
        loop {
            if let Some(message) = self.try_take_message()? {
                return Ok(message);
            }
            let mut chunk = [0u8; 4096];
            let stdout = self
                .child
                .stdout
                .as_mut()
                .ok_or_else(|| "vize lsp stdout is unavailable".to_string())?;
            let read = stdout
                .read(&mut chunk)
                .map_err(|error| format!("failed to read LSP frame: {error}"))?;
            if read == 0 {
                return Err("vize lsp closed stdout".to_string());
            }
            self.buffer.extend_from_slice(&chunk[..read]);
        }
    }

    fn try_take_message(&mut self) -> Result<Option<Value>, String> {
        let Some(header_end) = find_bytes(&self.buffer, b"\r\n\r\n") else {
            return Ok(None);
        };
        let header = String::from_utf8_lossy(&self.buffer[..header_end]);
        let mut length = None;
        for line in header.lines() {
            if let Some(value) = line.strip_prefix("Content-Length:") {
                length = Some(
                    value
                        .trim()
                        .parse::<usize>()
                        .map_err(|error| format!("invalid Content-Length: {error}"))?,
                );
            }
        }
        let length = length.ok_or_else(|| "LSP frame missing Content-Length".to_string())?;
        let body_start = header_end + 4;
        if self.buffer.len() < body_start + length {
            return Ok(None);
        }
        let body = self.buffer[body_start..body_start + length].to_vec();
        self.buffer.drain(..body_start + length);
        serde_json::from_slice(&body)
            .map(Some)
            .map_err(|error| format!("invalid LSP JSON: {error}"))
    }
}

pub fn file_url(path: &Path) -> Result<String, String> {
    let absolute = path
        .canonicalize()
        .map_err(|error| format!("cannot canonicalize {}: {error}", path.display()))?;
    Ok(format!(
        "file://{}",
        percent_encode_path(&absolute.display().to_string())
    ))
}

pub fn assert_json_eq(actual: &Value, expected: Value, label: &str) -> Result<(), String> {
    if actual != &expected {
        return Err(format!(
            "{label} mismatch\nexpected: {}\nactual: {}",
            serde_json::to_string_pretty(&expected).unwrap_or_else(|_| expected.to_string()),
            serde_json::to_string_pretty(actual).unwrap_or_else(|_| actual.to_string())
        ));
    }
    Ok(())
}

pub fn run_editor_contract(repo_root: &Path, label: &str, formatting: bool) -> Result<(), String> {
    let session_root = unique_temp_dir(&format!("vize-{label}-e2e"))?;
    let workspace_path = session_root.join("real-vue");
    editor_e2e::prepare_real_vue_workspace(&workspace_path, false)?;
    let mut session = LspSession::spawn(repo_root)?;
    let result = (|| {
        let mut options = json!({
            "editor": true,
            "ecosystem": true,
            "lint": true,
            "typecheck": true
        });
        if formatting {
            options["formatting"] = json!(true);
        }
        let initialization = session.initialize(&workspace_path, options)?;
        let document_formatting_provider =
            initialization.pointer("/capabilities/documentFormattingProvider");
        if formatting {
            if document_formatting_provider != Some(&Value::Bool(true)) {
                return Err(format!(
                    "{label} initialization did not enable documentFormattingProvider"
                ));
            }
        } else if document_formatting_provider.is_some_and(|value| !value.is_null()) {
            return Err(format!(
                "{label} initialization exposed documentFormattingProvider without formatting"
            ));
        }
        let document_path = workspace_path.join("src/Scenario.vue");
        let uri = file_url(&document_path)?;
        let source = fs::read_to_string(&document_path)
            .map_err(|error| format!("cannot read {}: {error}", document_path.display()))?;
        let expected_source = "<script setup lang=\"ts\">\nimport Child from \"./Child.vue\";\n\nconst total = \"3\";\n</script>\n\n<template>\n<Child  :count=\"total\" />\n</template>\n";
        if source != expected_source {
            return Err("the shared editor fixture changed".to_string());
        }
        session.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": "vue",
                    "version": 1,
                    "text": source
                }
            }),
        )?;
        let diagnostics =
            session.wait_for_notification("textDocument/publishDiagnostics", |params| {
                params.get("uri").and_then(Value::as_str) == Some(uri.as_str())
                    && params
                        .get("diagnostics")
                        .and_then(Value::as_array)
                        .is_some_and(|diagnostics| diagnostics.len() == 2)
            })?;
        assert_json_eq(
            diagnostics.get("diagnostics").unwrap_or(&Value::Null),
            expected_diagnostics(),
            "diagnostics",
        )?;
        let completion = session.request(
            "textDocument/completion",
            json!({
                "textDocument": { "uri": uri },
                "position": { "character": 16, "line": 7 }
            }),
        )?;
        assert_json_eq(&completion, expected_completion(), "completion")?;
        let hover = session.request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri },
                "position": { "character": 8, "line": 3 }
            }),
        )?;
        assert_json_eq(&hover, expected_hover(), "hover")?;
        let code_actions = session.request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri },
                "range": {
                    "end": { "character": 8, "line": 7 },
                    "start": { "character": 6, "line": 7 }
                },
                "context": { "diagnostics": expected_diagnostics() }
            }),
        )?;
        assert_json_eq(&code_actions, expected_code_actions(&uri), "code actions")?;
        if formatting {
            let response = session.request(
                "textDocument/formatting",
                json!({
                    "textDocument": { "uri": uri },
                    "options": { "insertSpaces": true, "tabSize": 2 }
                }),
            )?;
            assert_json_eq(&response, expected_formatting(), "formatting")?;
        }
        let semantic_tokens = session.request(
            "textDocument/semanticTokens/full",
            json!({ "textDocument": { "uri": uri } }),
        )?;
        assert_json_eq(
            &semantic_tokens,
            json!({ "data": [7, 8, 6, 9, 0, 0, 8, 5, 8, 0] }),
            "semantic tokens",
        )?;
        let rename = session.request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri },
                "position": { "character": 8, "line": 3 },
                "newName": "quantity"
            }),
        )?;
        assert_json_eq(&rename, expected_rename(&uri), "rename")?;
        Ok(())
    })();
    let shutdown = session.shutdown();
    let _ = fs::remove_dir_all(&session_root);
    result.and(shutdown)?;
    println!(
        "{}",
        if formatting {
            "zed extension-contract real-server scenario passed"
        } else {
            "helix package-contract real-server scenario passed"
        }
    );
    Ok(())
}

fn expected_diagnostics() -> Value {
    json!([
        {
            "code": "vue/no-multi-spaces",
            "codeDescription": { "href": "https://eslint.vuejs.org/rules/no-multi-spaces.html" },
            "message": "Multiple consecutive spaces",
            "range": {
                "end": { "character": 8, "line": 7 },
                "start": { "character": 6, "line": 7 }
            },
            "severity": 2,
            "source": "vize/lint"
        },
        {
            "code": 2322,
            "message": "Type 'string' is not assignable to type 'number'.",
            "range": {
                "end": { "character": 14, "line": 7 },
                "start": { "character": 9, "line": 7 }
            },
            "severity": 1,
            "source": "vize/types"
        }
    ])
}

fn expected_completion() -> Value {
    json!([
        {
            "detail": " (const)",
            "documentation": {
                "kind": "markdown",
                "value": "**Const**\n\nConstant binding (function, class, or literal)."
            },
            "kind": 21,
            "label": "Child",
            "labelDetails": { "detail": " (const)" },
            "sortText": "0Child"
        },
        {
            "detail": " (literal)",
            "documentation": {
                "kind": "markdown",
                "value": "**Literal**\n\nLiteral constant value."
            },
            "kind": 21,
            "label": "total",
            "labelDetails": { "detail": " (literal)" },
            "sortText": "0total"
        }
    ])
}

fn expected_hover() -> Value {
    json!({
        "contents": {
            "kind": "markdown",
            "value": "```typescript\nconst total: \"3\"\n```"
        },
        "range": {
            "end": { "character": 11, "line": 3 },
            "start": { "character": 6, "line": 3 }
        }
    })
}

fn expected_code_actions(uri: &str) -> Value {
    let mut changes = Map::new();
    changes.insert(
        uri.to_string(),
        json!([{
            "newText": " ",
            "range": {
                "end": { "character": 8, "line": 7 },
                "start": { "character": 6, "line": 7 }
            }
        }]),
    );
    let mut suppression_changes = Map::new();
    suppression_changes.insert(
        uri.to_string(),
        json!([{
            "newText": "<!-- @vize:forget vue/no-multi-spaces -->\n",
            "range": {
                "end": { "character": 0, "line": 7 },
                "start": { "character": 0, "line": 7 }
            }
        }]),
    );
    json!([
        {
            "edit": {
                "changes": Value::Object(changes)
            },
            "isPreferred": true,
            "kind": "quickfix",
            "title": "Fix: Replace multiple spaces with single space"
        },
        {
            "edit": {
                "changes": Value::Object(suppression_changes)
            },
            "isPreferred": false,
            "kind": "quickfix",
            "title": "Suppress with @vize:forget (vue/no-multi-spaces)"
        }
    ])
}

fn expected_formatting() -> Value {
    json!([
        {
            "newText": "<script setup lang=\"ts\">\nimport Child from \"./Child.vue\";\n\nconst total = \"3\";\n</script>\n\n<template>\n  <Child :count=\"total\" />\n</template>\n",
            "range": {
                "end": { "character": 0, "line": 9 },
                "start": { "character": 0, "line": 0 }
            }
        }
    ])
}

fn expected_rename(uri: &str) -> Value {
    let mut changes = Map::new();
    changes.insert(
        uri.to_string(),
        json!([
            {
                "newText": "quantity",
                "range": {
                    "end": { "character": 11, "line": 3 },
                    "start": { "character": 6, "line": 3 }
                }
            },
            {
                "newText": "quantity",
                "range": {
                    "end": { "character": 21, "line": 7 },
                    "start": { "character": 16, "line": 7 }
                }
            }
        ]),
    );
    json!({
        "changes": Value::Object(changes)
    })
}

fn unique_temp_dir(prefix: &str) -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join(format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_nanos()
    ));
    fs::create_dir_all(&path)
        .map_err(|error| format!("cannot create {}: {error}", path.display()))?;
    Ok(path)
}

fn percent_encode_path(path: &str) -> String {
    let mut out = String::new();
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
