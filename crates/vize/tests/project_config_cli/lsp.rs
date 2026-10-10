use super::{
    lsp_process::{LspProcess, file_uri},
    support::{self, assert_no_dedicated_config, formatted, project, write},
};
use serde_json::{Value, json};

#[test]
fn lsp_initializes_project_formatting_without_options_and_respects_false() {
    for enabled in [true, false] {
        let project = project();
        let root = project.path();
        write(
            root,
            "vite.config.mjs",
            &support::SETTINGS.replace(", jsxTypecheck: true", ""),
        );
        if !enabled {
            write(
                root,
                "vite.config.mjs",
                &support::SETTINGS.replace("formatting: true", "formatting: false"),
            );
        }
        let expected = formatted(root, &[]);
        let uri = file_uri(&root.join("Quotes.vue"));
        let mut lsp = LspProcess::spawn(root);
        lsp.send(
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
                "processId":null,"rootUri":file_uri(root),"capabilities":{}
            }}),
        );
        let response = lsp.recv_response(1);
        assert!(response["result"].is_object(), "{response:#}");
        assert_eq!(
            response["result"]["capabilities"]["experimental"]["vize"],
            json!({"jsxTypecheck":true})
        );
        assert_eq!(
            response["result"]["capabilities"]["documentFormattingProvider"]
                .as_bool()
                .unwrap_or(false),
            enabled,
            "{response:#}"
        );
        lsp.send(json!({"jsonrpc":"2.0","method":"initialized","params":{}}));
        lsp.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{"uri":uri,"languageId":"vue","version":1,"text":support::QUOTES}
            }}),
        );
        let publication = lsp.recv_matching(|message| {
            message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == uri.as_str()
                && message["params"]["version"] == 1
        });
        assert_eq!(
            publication["params"]["diagnostics"],
            json!([]),
            "{publication:#}"
        );
        lsp.send(
            json!({"jsonrpc":"2.0","id":2,"method":"textDocument/formatting","params":{
                "textDocument":{"uri":uri},"options":{"tabSize":2,"insertSpaces":true}
            }}),
        );
        let response = lsp.recv_response(2);
        assert!(response["error"].is_null(), "{response:#}");
        if enabled {
            let edits = response["result"].as_array().expect("formatting edits");
            assert_eq!(edits.len(), 1, "{response:#}");
            assert_eq!(edits[0]["newText"], expected.as_str(), "{response:#}");
            assert_eq!(expected.as_str(), support::PRESERVED_SINGLE);
        } else {
            assert_eq!(response["result"], Value::Null, "{response:#}");
        }
        lsp.send(json!({"jsonrpc":"2.0","id":3,"method":"shutdown"}));
        assert!(lsp.recv_response(3)["result"].is_null());
        lsp.send(json!({"jsonrpc":"2.0","method":"exit"}));
        assert!(lsp.wait_for_exit().success());
        assert_no_dedicated_config(root);
    }
}
