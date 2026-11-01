//! Read-only formatter requests must expose conflicting lint policy explicitly.

use super::*;
use serde_json::{Value, json};
use std::{
    io::{self, Write},
    path::Path,
    sync::{Arc, Mutex},
};

const RULE: &str = "vue/v-bind-style";
const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/cases.json"
));

fn original_fixed() -> std::string::String {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let expected = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "original-longform")
        .unwrap()["reference"]["fixResult"]["output"]
        .as_str()
        .unwrap()
        .to_owned();
    let pinned = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp/formatter-bind-style-policy-6098/Fixed.vue.txt"
    ));
    assert_eq!(pinned, expected);
    pinned.to_owned()
}

fn configure(server: &MaestroServer, root: &Path, configuration: Value) {
    let source = vize_l0::cstr!("export default {{ vue: {configuration} }};\n");
    std::fs::write(root.join("vite.config.mjs"), source.as_bytes()).unwrap();
    server.state.load_format_config(root);
}

fn policy(normalize: Option<bool>) -> Value {
    let mut value = json!({"linter":{"preset":"incremental","rules":{RULE:"error"},"ruleOptions":{RULE:"longform"}}});
    if let Some(normalize) = normalize {
        value["formatter"] = json!({"normalizeDirectiveShorthands":normalize});
    }
    value
}

fn selection(uri: Url) -> DocumentRangeFormattingParams {
    let mut value = range_params(uri);
    value.range = Range::new(Position::new(0, 12), Position::new(0, 40));
    value
}

fn on_type(uri: Url) -> DocumentOnTypeFormattingParams {
    let mut value = on_type_params(uri);
    value.text_document_position.position = Position::new(0, 24);
    value.ch = "\n".into();
    value
}

fn requests(server: &MaestroServer, uri: &Url) -> [Option<Vec<TextEdit>>; 3] {
    [
        futures::executor::block_on(server.formatting(formatting_params(uri.clone()))).unwrap(),
        futures::executor::block_on(server.range_formatting(selection(uri.clone()))).unwrap(),
        futures::executor::block_on(server.on_type_formatting(on_type(uri.clone()))).unwrap(),
    ]
}

#[derive(Clone)]
struct LogWriter(Arc<Mutex<Vec<u8>>>);

impl Write for LogWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn every_handler_logs_the_conflict_returns_null_and_keeps_authored_bytes() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    let file = root.join("VBindProp.vue");
    let source = original_fixed();
    std::fs::write(&file, &source).unwrap();
    let uri = Url::from_file_path(&file).unwrap();
    let service = server_with(&uri, &source, "vue");
    let server = service.inner();
    for normalize in [None, Some(true)] {
        configure(server, root, policy(normalize));
        if normalize == Some(true) {
            assert_eq!(
                std::fs::read(root.join("vite.config.mjs")).unwrap(),
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/_fixtures/differential/lsp/formatter-bind-style-policy-6098/vite.config.mjs.txt"
                ))
            );
        }
        let config_before = std::fs::read(root.join("vite.config.mjs")).unwrap();
        let logs = Arc::new(Mutex::new(Vec::new()));
        let writer = LogWriter(Arc::clone(&logs));
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_max_level(tracing::Level::WARN)
            .with_writer(move || writer.clone())
            .finish();
        let result = tracing::subscriber::with_default(subscriber, || requests(server, &uri));
        assert_eq!(result, [None, None, None]);
        let logs = std::string::String::from_utf8(logs.lock().unwrap().clone()).unwrap();
        assert_eq!(logs.matches("Refusing formatter request").count(), 3);
        assert_eq!(
            logs.matches(
                "normalizeDirectiveShorthands=true conflicts with active vue/v-bind-style=longform"
            )
            .count(),
            3
        );
        assert!(logs.contains("formatter.normalizeDirectiveShorthands=false"));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
        assert_eq!(
            std::fs::read(root.join("vite.config.mjs")).unwrap(),
            config_before
        );
        assert_eq!(
            server.state.documents.text_snapshot(&uri).unwrap().as_str(),
            source
        );
    }

    configure(server, root, policy(Some(false)));
    let result = requests(server, &uri);
    assert!(result.iter().all(Option::is_some));
    let options = vize_glyph::FormatOptions {
        normalize_directive_shorthands: false,
        ..Default::default()
    };
    let formatted = vize_glyph::format_sfc(&source, &options).unwrap().code;
    let full = result[0].as_ref().unwrap();
    assert_eq!(full.len(), 1);
    assert_eq!(full[0].new_text, formatted.as_str());
    assert!(formatted.contains("v-bind:title.prop="));
    server
        .state
        .documents
        .open(uri.clone(), formatted.to_string(), 2, "vue".into());
    let full =
        futures::executor::block_on(server.formatting(formatting_params(uri.clone()))).unwrap();
    assert_eq!(full, Some(Vec::new()));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
    assert_eq!(
        server.state.documents.text_snapshot(&uri).unwrap().as_str(),
        formatted.as_str()
    );
}

#[test]
fn each_formatter_request_resolves_entry_order_negations_and_inactive_files() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    let source = original_fixed();
    let uri = Url::from_file_path(root.join("Root.vue")).unwrap();
    let service = server_with(&uri, &source, "vue");
    let server = service.inner();
    configure(
        server,
        root,
        json!({
            "ignores":["ignored/**"],
            "linter":{"preset":"incremental","rules":{RULE:"off"},"ruleOptions":{RULE:"longform"}},
            "entries":[
                {"files":["src/**", "!src/Keep.vue"],"linter":{"rules":{RULE:"error"}}},
                {"files":["src/final/**"],"linter":{"rules":{RULE:"off"}}},
                {"basePath":"packages/ui","files":["**/*.vue"],"linter":{"rules":{RULE:"error"}}},
                {"files":["src/short/**"],"linter":{"ruleOptions":{RULE:"shorthand"}}}
            ]
        }),
    );
    for (name, conflict) in [
        ("src/Active.vue", true),
        ("src/Keep.vue", false),
        ("src/final/Off.vue", false),
        ("src/short/Short.vue", false),
        ("outside/Card.vue", false),
        ("packages/ui/Card.vue", true),
        ("ignored/Card.vue", false),
    ] {
        let file = root.join(name);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, &source).unwrap();
        let uri = Url::from_file_path(&file).unwrap();
        server
            .state
            .documents
            .open(uri.clone(), source.clone(), 1, "vue".into());
        let result = requests(server, &uri);
        assert_eq!(result.iter().all(Option::is_none), conflict, "{name}");
        if !conflict {
            assert!(result.iter().all(Option::is_some), "{name}");
        }
        assert_eq!(std::fs::read_to_string(&file).unwrap(), source, "{name}");
        assert_eq!(
            server.state.documents.text_snapshot(&uri).unwrap().as_str(),
            source,
            "{name}"
        );
    }
}
