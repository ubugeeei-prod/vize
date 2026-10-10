//! The native editor consumes the canonical ordered Vite-root ignore matcher.
use super::{SOURCE, Value, cstr, file_uri, finish, fixture, initialize, json, native, write};

const IMAGE: &str = "<template>\n  <img src=\"x\" />\n</template>\n";
const CONFIG: &str = r#"export default {root:'src',vize:{ignores:['generated/**','!generated/KeepItem.vue','pages/\\[id\\].vue'],linter:{preset:'essential',rules:{'a11y/alt-text':'error'}},lsp:{lint:true,typecheck:false}}};"#;

fn alt_text() -> Value {
    json!([{
        "range":{"start":{"line":1,"character":2},"end":{"line":1,"character":17}},
        "severity":1,"code":"a11y/alt-text",
        "codeDescription":{"href":"https://eslint.vuejs.org/rules/a11y/alt-text.html"},
        "source":"vize/lint",
        "message":"<img> elements must have an alt attribute\n\nHelp: Add an alt attribute: <img alt=\"Description\"> or <img alt=\"\"> for decorative images"
    }])
}

#[test]
fn vite_root_global_ignores_preserve_negation_escaping_and_watch_diagnostics() {
    let project = fixture(None);
    let root = project.path();
    write(root, "packages/a/vite.config.mjs", CONFIG);
    let mut lsp = initialize(root, Value::Null);
    for (file, ignored) in [
        ("generated/SkippedItem.vue", true),
        ("generated/KeepItem.vue", false),
        ("pages/[id].vue", true),
        ("pages/i.vue", false),
    ] {
        let file = cstr!("packages/a/src/{file}");
        write(root, &file, IMAGE);
        let uri = file_uri(&root.join(file));
        lsp.send(
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{"uri":uri,"languageId":"vue","version":1,"text":IMAGE}
            }}),
        );
        let publication = lsp.recv_matching(|message| {
            message["method"] == "textDocument/publishDiagnostics"
                && message["params"]["uri"] == uri.as_str()
                && message["params"]["version"] == 1
        });
        assert_eq!(
            publication["params"]["diagnostics"],
            if ignored { json!([]) } else { alt_text() },
            "{publication:#}"
        );
    }
    let config = root.join("packages/a/vite.config.mjs");
    std::fs::write(&config, CONFIG.replace("'!generated/KeepItem.vue',", "")).unwrap();
    lsp.send(
        json!({"jsonrpc":"2.0","method":"workspace/didChangeWatchedFiles","params":{
            "changes":[{"uri":file_uri(&config),"type":2}]
        }}),
    );
    let uri = file_uri(&root.join("packages/a/src/generated/KeepItem.vue"));
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
    finish(lsp);
}

#[test]
fn ignored_open_sources_keep_native_types_while_lint_exclusions_and_negation_apply() {
    let Some(runtime) = native::runtime() else {
        return;
    };
    let project = fixture(Some(&runtime));
    let root = project.path();
    super::super::typecheck::link_vue(root);
    let path = root.join("packages/a/vite.config.mjs");
    let settings = std::fs::read_to_string(&path)
        .unwrap()
        .replace(
            "lint:false",
            "lint:true",
        )
        .replace(
            "vize:{",
            "vize:{ignores:['generated/**','!generated/KeepItem.vue'],linter:{preset:'essential',rules:{'a11y/alt-text':'error'}},",
        );
    std::fs::write(path, settings).unwrap();
    let source = SOURCE.replace("VALUE_TYPE", "string").replace(
        "<template><p>{{ value }} {{ message }}</p></template>",
        IMAGE.trim_end(),
    );
    let mut lsp = initialize(root, Value::Null);
    for (name, ignored) in [("SkippedItem", true), ("KeepItem", false)] {
        let file = cstr!("packages/a/src/generated/{name}.vue");
        write(root, &file, &source);
        let uri = file_uri(&root.join(file));
        lsp.send(json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":uri,"languageId":"vue","version":1,"text":source}}}));
        let mismatch = native::mismatch("number", "string", 2, "value");
        let expected = if ignored {
            mismatch
        } else {
            let mut lint = alt_text();
            lint[0]["range"]["start"]["line"] = json!(6);
            lint[0]["range"]["end"]["line"] = json!(6);
            let mut diagnostics = lint.as_array().unwrap().clone();
            diagnostics.extend(mismatch.as_array().unwrap().iter().cloned());
            json!(diagnostics)
        };
        native::publication(&mut lsp, &uri, 1, &expected);
    }
    finish(lsp);
}
