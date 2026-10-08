//! Every fresh rename fixture must prove the actual native checker is active.

use serde_json::{Value, json};

use super::Fixture;

pub(super) fn prove(fixture: &mut Fixture) {
    let invalid = "<script setup lang=\"ts\">\nconst nativeGuard: number = 'wrong';\n</script>\n<template>{{ nativeGuard }}</template>\n";
    let repaired = "<script setup lang=\"ts\">\nconst nativeGuard: number = 1;\n</script>\n<template>{{ nativeGuard }}</template>\n";
    std::fs::write(fixture.project.path().join("src/NativeGuard.vue"), invalid).unwrap();
    fixture.open_with_diagnostics(
        "src/NativeGuard.vue",
        invalid,
        1,
        json!([{
            "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":17}},
            "severity":1,"code":2322,"source":"vize/types",
            "message":"Type 'string' is not assignable to type 'number'."
        }]),
    );
    fixture.change_with_native_completion("src/NativeGuard.vue", repaired, 2, json!([]));
}

impl Fixture {
    pub(crate) fn recv_checked_with(
        &mut self,
        mut matches: impl FnMut(&Value, &mut std::collections::HashMap<(String, i64), Value>) -> bool,
    ) -> Value {
        let expected = &mut self.publications;
        let mismatches = &mut self.mismatches;
        self.lsp.recv_matching(|message| {
            if message["method"] == "textDocument/publishDiagnostics" {
                println!("whole publication: {message}");
                let _: lsp_types::PublishDiagnosticsParams =
                    serde_json::from_value(message["params"].clone())
                        .expect("typed whole publication");
                let key = (
                    message["params"]["uri"].as_str().unwrap().to_owned(),
                    message["params"]["version"].as_i64().unwrap(),
                );
                if expected.get(&key) != Some(message) {
                    mismatches.push(json!({"actual":message,"expected":expected.get(&key)}));
                }
            }
            matches(message, expected)
        })
    }

    pub(crate) fn recv_publication_sequence(
        &mut self,
        uri: &str,
        version: i64,
        expected_packets: &[Value],
    ) -> Vec<Value> {
        let mut packets = Vec::new();
        // One receive loop retains the original transaction deadline.
        self.recv_checked_with(|message, ledger| {
            if message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == uri
                && message["params"]["version"] == version
            {
                packets.push(message.clone());
                if let Some(next) = expected_packets.get(packets.len()) {
                    ledger.insert((uri.to_owned(), version), next.clone());
                } else {
                    return true;
                }
            }
            false
        });
        packets
    }
}

impl Fixture {
    pub(crate) fn change_with_native_completion(
        &mut self,
        file: &str,
        text: &str,
        version: i64,
        diagnostics: Value,
    ) {
        let uri = self.uri(file);
        let prompt = json!({
            "jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
            "params":{"uri":uri,"version":version,"diagnostics":[]}
        });
        let native = json!({
            "jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
            "params":{"uri":uri,"version":version,"diagnostics":diagnostics}
        });
        self.publications
            .insert((uri.clone(), version), prompt.clone());
        let expected_packets = [prompt, native];
        println!(
            "preauthored prompt/native publications: {}",
            json!(expected_packets)
        );
        self.lsp.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
                "textDocument":{"uri":uri,"version":version},"contentChanges":[{"text":text}]
            }}),
        );
        let packets = self.recv_publication_sequence(&uri, version, &expected_packets);
        assert_eq!(
            packets, expected_packets,
            "complete ordered prompt/native publications before the next source request"
        );
    }
}
