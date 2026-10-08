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

    pub(super) fn request(
        &mut self,
        method: &str,
        uri: &str,
        mut params: Value,
    ) -> TestResult<Value> {
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
        let result = response
            .get("result")
            .ok_or_else(|| format!("missing result: {response:#}"))?
            .clone();
        check(response, json!({"jsonrpc":"2.0","id":id,"result":result}))?;
        Ok(result)
    }

    pub(super) fn publication(
        &mut self,
        uri: &str,
        version: i64,
        diagnostics: Value,
    ) -> TestResult {
        let expected = json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
            "params":{"uri":uri,"version":version,"diagnostics":diagnostics}});
        // This target disables prompt lint feedback. Initial [] is withheld,
        // and didChange's versioned [] can only follow the awaited native
        // collection. Match the whole final result and retain all earlier
        // publications in the unchanged shared transport's terminal evidence.
        let response = self.process.recv_matching(|m| m == &expected);
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
