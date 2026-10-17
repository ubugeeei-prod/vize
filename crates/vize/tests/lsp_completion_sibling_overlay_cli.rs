//! A sibling's unchanged diagnostics must preserve a native completion's identity.
#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::string_slice,
    reason = "the real stdio fixture asserts authored strings and complete JSON messages"
)]

use std::path::Path;

use serde_json::{Value, json};
use vize_carton::corsa_resolver::discover_corsa_in_ancestors;

#[path = "support/lsp_process.rs"]
mod lsp_process;
use lsp_process::{LspProcess, file_uri};

const IMPORTER: &str = r#"<script setup lang="ts">
import Child from "./Child.vue";
const broken: number = "wrong";
</script>
<template><Child heading="April" />{{ broken }}</template>
"#;
const CHILD: &str = include_str!(
    "../../../editors/vscode/test-fixtures/extension-host/real-vue/src/InvoiceCard.vue"
);
const INDEPENDENT: &str = r#"<script setup lang="ts">
declare function makeTotals(): {
  /** **Invoice total** in minor currency units. */
  invoiceTotal: number;
};
const totals = makeTotals();
const broken: number = "wrong";
totals.invoiceTotal;
</script>
<template>{{ totals.invoiceTotal }} {{ broken }}</template>
"#;
const DETAIL: &str = "(property) invoiceTotal: number";
const MARKDOWN: &str = "```typescript\n(property) invoiceTotal: number\n```\n\n**Invoice total** in minor currency units.";

#[test]
fn native_completion_resolves_after_unchanged_sibling_diagnostics_with_a_closed_dependency() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let runtime = discover_corsa_in_ancestors(workspace)
        .expect("completion resolve regression requires the actual TypeScript native runtime");
    let vue = workspace
        .join("playground/node_modules/vue")
        .canonicalize()
        .expect("completion resolve regression requires the frozen Vue dependency");
    let project = tempfile::Builder::new()
        .prefix("lsp-completion-sibling-overlay-")
        .tempdir()
        .unwrap();
    let root = project.path();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("node_modules")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&vue, root.join("node_modules/vue")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&vue, root.join("node_modules/vue")).unwrap();
    for (name, source) in [
        ("src/Importer.vue", IMPORTER),
        ("src/Independent.vue", INDEPENDENT),
        ("src/Child.vue", CHILD),
    ] {
        std::fs::write(root.join(name), source).unwrap();
    }
    std::fs::write(
        root.join("tsconfig.json"),
        include_str!("../../../editors/vscode/test-fixtures/extension-host/real-vue/tsconfig.json"),
    )
    .unwrap();
    // Exercise the standard project-config path, with no dedicated Vize config.
    let options = json!({"typeChecker":{"corsaPath":runtime}});
    std::fs::write(
        root.join("vite.config.mjs"),
        format!("export default {{ vue: {options} }};\n"),
    )
    .unwrap();
    let importer = file_uri(&root.join("src/Importer.vue"));
    let independent = file_uri(&root.join("src/Independent.vue"));
    let mut lsp = LspProcess::spawn(root);
    let mut id = 0;
    let initialized = request(
        &mut lsp,
        &mut id,
        "initialize",
        json!({
            "processId":null,"rootUri":file_uri(root),
            "capabilities":{"textDocument":{"completion":{"completionItem":{
                "documentationFormat":["markdown"],"resolveSupport":{"properties":["documentation","detail"]}
            }}}},
            "initializationOptions":{"lint":false,"typecheck":true,"hover":true,"completion":true}
        }),
    );
    assert_eq!(
        initialized["capabilities"]["completionProvider"]["resolveProvider"],
        true
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));

    // A's closed dependency must already belong to the shared project view
    // when B performs its own native query. Child.vue is never opened.
    open_and_expect_native_diagnostics(&mut lsp, &importer, IMPORTER);
    open_and_expect_native_diagnostics(&mut lsp, &independent, INDEPENDENT);
    // A call initializer requires native return-type inference; it cannot use
    // the source-local object-literal completion shortcut.
    let completion = request(
        &mut lsp,
        &mut id,
        "textDocument/completion",
        json!({"textDocument":{"uri":independent},
            "position":position(INDEPENDENT, "invoiceTotal;"),"context":{"triggerKind":1}}),
    );
    let items = completion
        .as_array()
        .or_else(|| completion.get("items").and_then(Value::as_array))
        .unwrap_or_else(|| panic!("native completion list: {completion:#}"));
    let candidates = items
        .iter()
        .filter(|item| item["label"] == "invoiceTotal")
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1, "complete native list: {completion:#}");
    let issued = (*candidates[0]).clone();
    assert_eq!(issued["kind"], 5, "native field candidate: {issued:#}");
    assert_eq!(
        issued["data"]["vizeCompletion"]["uri"],
        independent.as_str()
    );
    assert!(
        issued["data"]["vizeCompletion"]["item"]["data"].is_object(),
        "a real native resolve identity is required: {issued:#}"
    );
    let documentation = json!({"kind":"markdown","value":MARKDOWN});
    assert_ne!(
        issued["documentation"], documentation,
        "documentation must still require resolve"
    );
    let opaque_identity = serde_json::to_vec(&issued["data"]).unwrap();

    // Save the exact same A bytes at the exact same document version. Its
    // native diagnostic publication is the completion-before-resolve barrier.
    lsp.send(
        json!({"jsonrpc":"2.0","method":"textDocument/didSave","params":{
            "textDocument":{"uri":importer},"text":IMPORTER
        }}),
    );
    expect_native_diagnostics(&mut lsp, &importer, IMPORTER);
    let resolved = request(&mut lsp, &mut id, "completionItem/resolve", issued.clone());
    let mut expected = issued;
    expected["detail"] = json!(DETAIL);
    expected["documentation"] = documentation;
    assert_eq!(
        resolved, expected,
        "the entire originally issued candidate must resolve"
    );
    assert_eq!(
        serde_json::to_vec(&resolved["data"]).unwrap(),
        opaque_identity
    );
    for (name, source) in [
        ("src/Importer.vue", IMPORTER),
        ("src/Independent.vue", INDEPENDENT),
        ("src/Child.vue", CHILD),
    ] {
        assert_eq!(std::fs::read_to_string(root.join(name)).unwrap(), source);
    }
    id += 1;
    lsp.send(json!({"jsonrpc":"2.0","id":id,"method":"shutdown"}));
    assert_eq!(
        lsp.recv_response(id),
        json!({"jsonrpc":"2.0","id":id,"result":null})
    );
    lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
    assert!(lsp.wait_for_exit().success());
}

fn request(lsp: &mut LspProcess, id: &mut i64, method: &str, params: Value) -> Value {
    *id += 1;
    lsp.send(json!({"jsonrpc":"2.0","id":*id,"method":method,"params":params}));
    let response = lsp.recv_response(*id);
    assert_eq!(
        response,
        json!({"jsonrpc":"2.0","id":*id,"result":response["result"]})
    );
    response["result"].clone()
}

fn open_and_expect_native_diagnostics(lsp: &mut LspProcess, uri: &str, source: &str) {
    lsp.send(
        json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
            "textDocument":{"uri":uri,"languageId":"vue","version":1,"text":source}
        }}),
    );
    expect_native_diagnostics(lsp, uri, source);
}

fn expect_native_diagnostics(lsp: &mut LspProcess, uri: &str, source: &str) {
    let response = lsp.recv_matching(|message| {
        message["method"] == "textDocument/publishDiagnostics"
            && message["params"]["uri"] == uri
            && message["params"]["version"] == 1
            && message["params"]["diagnostics"]
                .as_array()
                .is_some_and(|items| items.iter().any(|item| item["source"] == "vize/types"))
    });
    let start = position(source, "broken:");
    let mut end = start.clone();
    end["character"] = json!(start["character"].as_u64().unwrap() + "broken".len() as u64);
    assert_eq!(
        response,
        json!({"jsonrpc":"2.0","method":"textDocument/publishDiagnostics","params":{
            "uri":uri,"version":1,"diagnostics":[{
                "range":{"start":start,"end":end},"severity":1,"code":2322,"source":"vize/types",
                "message":"Type 'string' is not assignable to type 'number'."
            }]
        }})
    );
}

fn position(source: &str, needle: &str) -> Value {
    let prefix = &source[..source.find(needle).expect("authored fixture position")];
    let line = prefix.bytes().filter(|&byte| byte == b'\n').count();
    let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count();
    json!({"line":line,"character":column})
}
