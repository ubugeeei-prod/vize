#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

#[path = "support/lsp_process.rs"]
mod lsp_process;
use lsp_process::{LspProcess, file_uri};
use serde_json::{Value, json};
use std::path::Path;
use vize_carton::corsa_resolver::discover_corsa_in_ancestors;

#[path = "lsp_component_attribute_authority_cli/attribute_controls.rs"]
mod attribute_controls;
#[path = "lsp_library_rename_refusal_cli/importer_controls.rs"]
mod library_importer_controls;
#[path = "lsp_library_rename_refusal_cli/original.rs"]
mod library_refusal;
#[path = "lsp_component_attribute_authority_cli/native_probe.rs"]
mod native_probe;
#[path = "lsp_component_attribute_authority_cli/original_reads.rs"]
mod original_reads;

const ITEM: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/rename-source-authority/8009/Item.vue.txt"
);
const TYPES: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/rename-source-authority/8009/types.ts.txt"
);
const APP: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/rename-source-authority/8009/App.vue.txt"
);
const CLASSY: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/rename-source-authority/8009/Classy.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/rename-source-authority/8009/tsconfig.json.txt"
);

struct Fixture {
    project: tempfile::TempDir,
    lsp: LspProcess,
    id: i64,
    publications: std::collections::HashMap<(String, i64), Value>,
    mismatches: Vec<Value>,
    runtime: std::path::PathBuf,
    publication_sequences: Vec<Value>,
}

impl Fixture {
    fn new(files: &[(&str, &str)]) -> Self {
        Self::with_config(files, CONFIG)
    }

    fn with_config(files: &[(&str, &str)], config: &str) -> Self {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let runtime = discover_corsa_in_ancestors(workspace).expect("native runtime required");
        let cases = workspace.join("target/vize-tests/tests");
        std::fs::create_dir_all(&cases).unwrap();
        let project = tempfile::Builder::new()
            .prefix("lsp-rename-authority-")
            .tempdir_in(cases)
            .unwrap();
        for &(name, text) in files {
            let path = project.path().join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        std::fs::write(project.path().join("tsconfig.json"), config).unwrap();
        let modules = project.path().join("node_modules");
        std::fs::create_dir_all(&modules).unwrap();
        let vue = workspace
            .join("playground/node_modules/vue")
            .canonicalize()
            .expect("frozen Vue required");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&vue, modules.join("vue")).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&vue, modules.join("vue")).unwrap();
        std::fs::write(
            project.path().join("vize.config.json"),
            serde_json::to_vec(&json!({
                "typeChecker": { "corsaPath": runtime },
                "lsp": { "lint": true, "typecheck": true, "hover": true, "crossFile": true }
            }))
            .unwrap(),
        )
        .unwrap();
        let mut lsp = LspProcess::spawn(project.path());
        lsp.send(
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
                "processId":null,"rootUri":file_uri(project.path()),"capabilities":{},
                "initializationOptions":{"lint":true,"typecheck":true,"hover":true,"crossFile":true}
            }}),
        );
        assert!(lsp.recv_response(1)["result"].is_object());
        lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
        let mut fixture = Self {
            project,
            lsp,
            id: 2,
            publications: Default::default(),
            mismatches: Vec::new(),
            runtime,
            publication_sequences: Vec::new(),
        };
        native_probe::prove(&mut fixture);
        fixture
    }

    fn uri(&self, file: &str) -> String {
        file_uri(&self.project.path().join(file)).to_string()
    }

    fn open(&mut self, file: &str, text: &str, version: i64) {
        self.open_with_diagnostics(file, text, version, json!([]));
    }

    fn open_with_diagnostics(&mut self, file: &str, text: &str, version: i64, diagnostics: Value) {
        let uri = self.uri(file);
        self.expect_publication(&uri, version, diagnostics);
        self.lsp.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{"uri":uri,"languageId":"vue","version":version,"text":text}
            }}),
        );
        self.recv_checked(|m| {
            m["method"] == "textDocument/publishDiagnostics"
                && m["params"]["uri"] == uri
                && m["params"]["version"] == version
        });
    }

    fn change(&mut self, file: &str, text: &str, version: i64, diagnostics: Value) {
        let uri = self.uri(file);
        self.expect_publication(&uri, version, diagnostics);
        self.lsp.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didChange","params":{
                "textDocument":{"uri":uri,"version":version},"contentChanges":[{"text":text}]
            }}),
        );
        self.recv_checked(|m| {
            m["method"] == "textDocument/publishDiagnostics"
                && m["params"]["uri"] == uri
                && m["params"]["version"] == version
        });
    }

    fn expect_publication(&mut self, uri: &str, version: i64, diagnostics: Value) {
        self.publications.insert(
            (uri.to_owned(), version),
            json!({
                "jsonrpc":"2.0","method":"textDocument/publishDiagnostics",
                "params":{"uri":uri,"version":version,"diagnostics":diagnostics}
            }),
        );
    }

    fn recv_checked(&mut self, mut matches: impl FnMut(&Value) -> bool) -> Value {
        self.recv_checked_with(|message, _| matches(message))
    }

    fn request(&mut self, file: &str, method: &str, position: Value, extra: Value) -> Value {
        let id = self.id;
        self.id += 1;
        let mut params = extra;
        params["textDocument"] = json!({"uri":self.uri(file)});
        params["position"] = position;
        self.lsp
            .send(json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}));
        let response = self.recv_checked(|message| message["id"].as_i64() == Some(id));
        println!("original response {file} {method}: {response}");
        assert!(response.get("error").is_none(), "{response:#}");
        assert!(
            response.get("result").is_some(),
            "complete success response"
        );
        assert_eq!(
            response,
            json!({"jsonrpc":"2.0","id":id,"result":response["result"]})
        );
        response["result"].clone()
    }

    fn shutdown(&mut self) {
        let id = self.id;
        self.lsp
            .send(json!({"jsonrpc":"2.0","id":id,"method":"shutdown"}));
        assert_eq!(
            self.recv_checked(|message| message["id"].as_i64() == Some(id)),
            json!({"jsonrpc":"2.0","id":id,"result":null})
        );
        self.lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
        assert!(self.lsp.wait_for_exit().success());
        assert!(
            self.mismatches.is_empty(),
            "all whole publication mismatches: {:#?}",
            self.mismatches
        );
    }
}

#[test]
fn original_component_key_does_not_prepare_or_rename_define_emits() {
    let mut fixture = Fixture::new(&[
        ("src/Item.vue", ITEM),
        ("src/types.ts", TYPES),
        ("src/App.vue", APP),
        ("src/Classy.vue", CLASSY),
    ]);
    fixture.open("src/App.vue", APP, 1);
    for method in ["textDocument/prepareRename", "textDocument/rename"] {
        let result = fixture.request(
            "src/App.vue",
            method,
            json!({"line":15,"character":8}),
            json!({"newName":"keyZz"}),
        );
        assert_eq!(
            result,
            Value::Null,
            "whole-element mapping must not authorize a macro edit"
        );
    }
    fixture.open("src/Classy.vue", CLASSY, 1);
    let result = fixture.request(
        "src/Classy.vue",
        "textDocument/rename",
        json!({"line":3,"character":8}),
        json!({"newName":"showSuffix"}),
    );
    let edits = [
        "const hasSuffix",
        "v-if=\"hasSuffix",
        "{{ hasSuffix",
        "'with-gap': hasSuffix",
    ]
    .map(|prefix| {
        let offset = CLASSY.find(prefix).unwrap() + prefix.len() - "hasSuffix".len();
        json!({"range":range(CLASSY, offset, "hasSuffix"),"newText":"showSuffix"})
    });
    assert_eq!(
        result,
        json!({"changes":{fixture.uri("src/Classy.vue"):edits}}),
        "the legitimate binding must retain all four non-null authored edits"
    );
    let typed: lsp_types::WorkspaceEdit =
        serde_json::from_value(result).expect("non-null typed WorkspaceEdit");
    let mut applied = CLASSY.to_owned();
    let mut edits = typed
        .changes
        .unwrap()
        .into_values()
        .flatten()
        .collect::<Vec<_>>();
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.range.start));
    for edit in edits {
        let start = byte_offset(CLASSY, edit.range.start);
        let end = byte_offset(CLASSY, edit.range.end);
        applied.replace_range(start..end, &edit.new_text);
    }
    assert_eq!(applied, CLASSY.replace("hasSuffix", "showSuffix"));
    fixture.shutdown();
}

fn range(source: &str, offset: usize, token: &str) -> Value {
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|&b| b == b'\n').count();
    let start = prefix.rfind('\n').map_or(0, |n| n + 1);
    let character = prefix[start..].encode_utf16().count();
    json!({"start":{"line":line,"character":character},"end":{"line":line,"character":character+token.encode_utf16().count()}})
}

fn byte_offset(source: &str, position: lsp_types::Position) -> usize {
    let start = source
        .split_inclusive('\n')
        .take(position.line as usize)
        .map(str::len)
        .sum::<usize>();
    let line = source[start..].split('\n').next().unwrap();
    let mut units = 0;
    for (offset, ch) in line.char_indices() {
        if units == position.character {
            return start + offset;
        }
        units += ch.len_utf16() as u32;
        assert!(units <= position.character, "UTF-16 boundary");
    }
    assert_eq!(units, position.character);
    start + line.len()
}
