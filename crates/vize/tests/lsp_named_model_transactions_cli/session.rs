#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use super::{
    FILES, check,
    lsp_process::{LspProcess, file_uri},
};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use vize_carton::corsa_resolver::discover_corsa_in_ancestors;

const GUARD: &str = "<script setup lang=\"ts\">\nconst nativeGuard: number = 'wrong';\n</script>\n<template>{{ nativeGuard }}</template>\n";
const TSCONFIG: &str = include_str!("../fixtures/content_mapper_project/tsconfig.json");

pub struct Session {
    project: tempfile::TempDir,
    lsp: LspProcess,
    next_id: i64,
}

impl Session {
    pub fn new(case: &Value) -> Self {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let runtime = discover_corsa_in_ancestors(workspace)
            .expect("named-model editor contracts require the real workspace native runtime");
        let vue = workspace
            .join("playground/node_modules/vue")
            .canonicalize()
            .expect("named-model editor contracts require the frozen Playground Vue dependency");
        let root = workspace.join("target/vize-tests/tests");
        std::fs::create_dir_all(&root).unwrap();
        let project = tempfile::Builder::new()
            .prefix("lsp-named-model-")
            .tempdir_in(root)
            .unwrap();
        std::fs::create_dir_all(project.path().join("src")).unwrap();
        std::fs::create_dir_all(project.path().join("node_modules")).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&vue, project.path().join("node_modules/vue")).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&vue, project.path().join("node_modules/vue")).unwrap();
        std::fs::write(project.path().join("tsconfig.json"), TSCONFIG).unwrap();
        std::fs::write(project.path().join("vize.config.json"), serde_json::to_vec(&json!({
            "experimentals":{"patternedTemplate":false},
            "typeChecker":{"corsaPath":runtime,"checkFallthroughAttrs":false,"optionsApi":false},
            "lsp":{"lint":false,"typecheck":true,"hover":true,"crossFile":true},
        })).unwrap()).unwrap();
        for name in FILES {
            std::fs::write(
                project.path().join("src").join(name),
                case["sources"][name].as_str().unwrap(),
            )
            .unwrap();
        }
        std::fs::write(project.path().join("src/NativeGuard.vue"), GUARD).unwrap();
        let mut lsp = LspProcess::spawn(project.path());
        lsp.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "processId":null,"rootUri":file_uri(project.path()),"capabilities":{},
            "initializationOptions":{"lint":false,"typecheck":true,"hover":true,"crossFile":true},
        }}));
        assert!(lsp.recv_response(1)["result"].is_object());
        lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
        Self {
            project,
            lsp,
            next_id: 2,
        }
    }

    pub fn uri(&self, name: &str) -> String {
        file_uri(&self.project.path().join("src").join(name)).to_string()
    }

    pub fn uris(&self) -> Value {
        json!(
            FILES
                .iter()
                .map(|name| (name.to_string(), json!(self.uri(name))))
                .collect::<serde_json::Map<_, _>>()
        )
    }

    pub fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.project.path().join("src").join(name)).unwrap()
    }

    pub fn publication(&self, name: &str, version: i64, diagnostics: Value) -> Value {
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{"uri":self.uri(name),"version":version,"diagnostics":diagnostics}})
    }

    fn guard_diagnostic(&self) -> Value {
        json!([{
            "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":17}},
            "severity":1,"code":2322,"source":"vize/types",
            "message":"Type 'string' is not assignable to type 'number'.",
        }])
    }

    pub fn expected_publications(&self, case: &Value) -> Vec<Value> {
        let mut allowed = vec![
            self.publication("NativeGuard.vue", 1, self.guard_diagnostic()),
            self.publication("NativeGuard.vue", 2, json!([])),
        ];
        for name in FILES {
            allowed.push(self.publication(name, 1, json!([])));
            allowed.push(self.publication(
                name,
                2,
                case["expectedVersion2Diagnostics"][name].clone(),
            ));
            allowed.push(self.publication(
                name,
                3,
                case["expectedIndependentVersion3Diagnostics"][name].clone(),
            ));
        }
        allowed
    }

    fn receive_publication(&mut self, name: &str, version: i64) -> Value {
        let uri = self.uri(name);
        self.lsp.recv_matching(|message| {
            message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == uri
                && message["params"]["version"] == version
        })
    }

    pub fn open(&mut self, name: &str, text: &str) -> Value {
        self.lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":self.uri(name),"languageId":"vue","version":1,"text":text}}}));
        self.receive_publication(name, 1)
    }

    fn change(&mut self, name: &str, text: &str, version: i64) -> Value {
        self.lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{"textDocument":{"uri":self.uri(name),"version":version},"contentChanges":[{"text":text}]}}));
        self.receive_publication(name, version)
    }

    pub fn prove_native_runtime(&mut self, failures: &mut Vec<Value>) -> Value {
        let expected = self.publication("NativeGuard.vue", 1, self.guard_diagnostic());
        let initial = self.open("NativeGuard.vue", GUARD);
        check(
            failures,
            "mandatory whole native guard TS2322",
            &initial,
            &expected,
        );
        let repaired = GUARD.replace("'wrong'", "1");
        std::fs::write(self.project.path().join("src/NativeGuard.vue"), &repaired).unwrap();
        let changed = self.change("NativeGuard.vue", &repaired, 2);
        check(
            failures,
            "mandatory whole native guard repair",
            &changed,
            &self.publication("NativeGuard.vue", 2, json!([])),
        );
        json!({"invalidDiagnostics":initial,"repairedDiagnostics":changed,"expectedInvalid":expected})
    }

    pub fn query(&mut self, request: &Value) -> Value {
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["id"], self.next_id);
        self.next_id += 1;
        self.lsp.send(request.clone());
        self.lsp.recv_response(self.next_id - 1)
    }

    pub fn install(&mut self, files: &Value, version: i64) -> Value {
        // Install every real returned-edit file before publishing any change.
        for name in FILES {
            std::fs::write(
                self.project.path().join("src").join(name),
                files[name].as_str().unwrap(),
            )
            .unwrap();
        }
        json!(FILES.iter().map(|name| {
            let text = files[name].as_str().unwrap();
            let diagnostics = self.change(name, text, version);
            json!({"file":name,"text":text,"disk":self.read(name),"version":version,"diagnostics":diagnostics})
        }).collect::<Vec<_>>())
    }

    pub fn expected_files(&self, files: &Value, version: i64) -> Value {
        json!(FILES.iter().map(|name| json!({"file":name,"text":files[name],"disk":files[name],"version":version,"diagnostics":self.publication(name, version, json!([]))})).collect::<Vec<_>>())
    }

    pub fn finish(&mut self) {
        let id = self.next_id;
        self.lsp
            .send(json!({"jsonrpc":"2.0","id":id,"method":"shutdown"}));
        let reply = self.lsp.recv_response(id);
        assert_eq!(reply, json!({"jsonrpc":"2.0","id":id,"result":null}));
        self.lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
        assert!(self.lsp.wait_for_exit().success());
    }

    pub fn publications(&self) -> Vec<Value> {
        self.lsp.published_diagnostics()
    }

    pub fn capture(&self, context: &str, oracle: &Value, expected: &Value, actual: &Value) {
        let root = std::env::var_os("VIZE_TEST_FIX_HISTORY_CAPTURE_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                Some(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .parent()?
                        .parent()?
                        .join("target/nextest")
                        .join(std::env::var_os("NEXTEST_PROFILE")?),
                )
            });
        let Some(root) = root else {
            return;
        };
        let root = root.join("named-model-transactions");
        std::fs::create_dir_all(&root).unwrap();
        let packet = json!({"context":context,"sourceSha":std::env::var("SOURCE_SHA").ok(),
            "cliBinary":env!("CARGO_BIN_EXE_vize"),"requireTsgo":std::env::var("VIZE_TEST_REQUIRE_TSGO").ok(),
            "disableTsgo":std::env::var("VIZE_TEST_DISABLE_TSGO").ok(),"oracle":oracle,"expected":expected,"actual":actual,
            "tsconfig":TSCONFIG,"vizeConfig":std::fs::read_to_string(self.project.path().join("vize.config.json")).unwrap(),
            "rawTraceDir":self.lsp.trace_dir(),"allDefinitionOriginsContracted":true});
        std::fs::write(
            root.join(self.project.path().file_name().unwrap())
                .with_extension("json"),
            serde_json::to_vec_pretty(&packet).unwrap(),
        )
        .unwrap();
    }
}
