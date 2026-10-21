//! Unmodified shared Content-Length transport for two independent executables.
use std::path::Path;

use serde_json::{Value, json};

use super::lsp_process::{LspProcess, file_uri};
use super::{TestResult, check};

pub(super) struct Session {
    process: LspProcess,
    id: i64,
    native: bool,
}

impl Session {
    pub(super) fn new(root: &Path, executable: Option<&Path>) -> TestResult<Self> {
        let native = executable.is_some();
        let mut process = match executable {
            Some(executable) => LspProcess::spawn_native(root, executable),
            None => LspProcess::spawn(root),
        };
        process.retain_protocol();
        let options = if native {
            json!({"userPreferences":{"tsserver":{"automaticTypeAcquisition":{"enabled":false}}}})
        } else {
            json!({"lint":false,"typecheck":true,"hover":true,"crossFile":true})
        };
        process.send(
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
                "processId":null,"rootUri":file_uri(root),
                "capabilities":{"textDocument":{"diagnostic":{"dynamicRegistration":false,
                    "relatedDocumentSupport":true,"relatedInformation":true}}},
                "initializationOptions":options
            }}),
        );
        let response = process.recv_response(1);
        let result = response
            .get("result")
            .ok_or("initialization result missing")?
            .clone();
        if !result.is_object() {
            return Err(format!("initialization result is not an object: {response:#}").into());
        }
        check(response, json!({"jsonrpc":"2.0","id":1,"result":result}))?;
        process.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
        if native {
            let registration = process
                .recv_matching(|m| m.get("method") == Some(&json!("client/registerCapability")));
            check(
                registration,
                json!({"jsonrpc":"2.0","id":"ts1",
                "method":"client/registerCapability","params":{"registrations":[{
                    "id":"typescript-config-watch-id","method":"workspace/didChangeConfiguration",
                    "registerOptions":{"section":["js/ts","typescript","javascript","editor"]}
                }]}}),
            )?;
            process.send(json!({"jsonrpc":"2.0","id":"ts1","result":null}));
        }
        Ok(Self {
            process,
            id: 2,
            native,
        })
    }

    pub(super) fn open(&mut self, uri: &str, source: &str, language: &str, version: i64) {
        self.process.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{"uri":uri,"languageId":language,"version":version,"text":source}
            }}),
        );
    }

    pub(super) fn change(&mut self, uri: &str, source: &str, version: i64) {
        self.process.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
                "textDocument":{"uri":uri,"version":version},"contentChanges":[{"text":source}]
            }}),
        );
    }

    pub(super) fn close(&mut self, uri: &str) {
        self.process.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didClose","params":{
                "textDocument":{"uri":uri}
            }}),
        );
    }

    pub(super) fn request(&mut self, method: &str, uri: &str, params: Value) -> TestResult<Value> {
        let (id, response) = self.request_response(method, uri, params)?;
        response_result(id, response)
    }

    /// Read-only navigation can race a newer names snapshot after an editor
    /// lifecycle transition. LSP asks clients to retry `ContentModified`;
    /// preserve that refusal in the protocol evidence and retry exactly once.
    pub(super) fn navigation_request(
        &mut self,
        method: &str,
        uri: &str,
        params: Value,
    ) -> TestResult<Value> {
        for attempt in 0..=1 {
            let (id, response) = self.request_response(method, uri, params.clone())?;
            if should_retry_navigation(self.native, attempt, &response) {
                continue;
            }
            return response_result(id, response);
        }
        unreachable!("the second navigation response is always terminal")
    }

    fn request_response(
        &mut self,
        method: &str,
        uri: &str,
        mut params: Value,
    ) -> TestResult<(i64, Value)> {
        params
            .as_object_mut()
            .ok_or("request parameters must be an object")?
            .insert("textDocument".into(), json!({"uri":uri}));
        let id = self.id;
        self.id += 1;
        self.process
            .send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        let response = self.process.recv_response(id);
        println!(
            "whole {} response: {response}",
            if self.native { "stock native" } else { "Vize" }
        );
        Ok((id, response))
    }

    pub(super) fn publication(
        &mut self,
        uri: &str,
        version: i64,
        diagnostics: Value,
    ) -> TestResult {
        let expected = json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
            "params":{"uri":uri,"version":version,"diagnostics":diagnostics}});
        // This target disables prompt lint feedback. Initial [] is withheld;
        // the versioned native result may include actual authored errors.
        // Match the whole final result and retain all earlier publications in
        // the unchanged shared transport's terminal evidence.
        let response = self.process.recv_matching(|m| m == &expected);
        check(response, expected)
    }

    pub(super) fn closed_publication(&mut self, uri: &str) -> TestResult {
        let expected = json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
            "params":{"uri":uri,"diagnostics":[]}});
        let response = self.process.recv_matching(|message| message == &expected);
        check(response, expected)
    }

    pub(super) fn finish(&mut self, executable: &Path) -> TestResult {
        let id = self.id;
        self.process
            .send(json!({"jsonrpc":"2.0","id":id,"method":"shutdown"}));
        check(
            self.process.recv_response(id),
            json!({"jsonrpc":"2.0","id":id,"result":null}),
        )?;
        let status = if self.native {
            self.process.wait_for_transport_eof()
        } else {
            self.process.send(json!({"jsonrpc":"2.0","method":"exit"}));
            self.process.wait_for_exit()
        };
        if !status.success() {
            return Err(format!("stdio process exited unsuccessfully: {status}").into());
        }
        // The shared helper's executable field describes its integration binary.
        // Pair the unchanged complete evidence with the actual native executable.
        println!(
            "terminal stdio custody: {}",
            json!({
                "route":if self.native {"stock native"} else {"Vize"},
                "actualExecutable":if self.native {executable} else {Path::new(env!("CARGO_BIN_EXE_vize"))},
                "completeEvidence":self.process.terminal_evidence()
            })
        );
        Ok(())
    }
}

fn response_result(id: i64, response: Value) -> TestResult<Value> {
    let result = response
        .get("result")
        .ok_or_else(|| format!("missing result: {response:#}"))?
        .clone();
    check(response, json!({"jsonrpc":"2.0","id":id,"result":result}))?;
    Ok(result)
}

fn should_retry_navigation(native: bool, attempt: u8, response: &Value) -> bool {
    !native
        && attempt == 0
        && response.pointer("/error/code").and_then(Value::as_i64) == Some(-32801)
}

#[cfg(test)]
mod tests {
    use super::should_retry_navigation;
    use serde_json::json;

    #[test]
    fn only_the_first_vize_content_modified_navigation_is_retryable() {
        let modified = json!({
            "jsonrpc":"2.0","id":2,"error":{"code":-32801,"message":"Content modified"}
        });
        assert!(should_retry_navigation(false, 0, &modified));
        assert!(!should_retry_navigation(false, 1, &modified));
        assert!(!should_retry_navigation(true, 0, &modified));
        assert!(!should_retry_navigation(
            false,
            0,
            &json!({
                "jsonrpc":"2.0","id":2,"error":{"code":-32800,"message":"Request cancelled"}
            })
        ));
        assert!(!should_retry_navigation(
            false,
            0,
            &json!({"jsonrpc":"2.0","id":2,"result":null})
        ));
    }
}
