//! New event-policy interaction; original bind-policy controls stay unchanged.

use super::*;
use serde_json::{Value, json};
use std::{cell::RefCell, path::Path};
use tower_lsp::lsp_types::NumberOrString;
use tracing_subscriber::{Layer, layer::SubscriberExt};
use vize_l0::{String, cstr};

const RULE: &str = "vue/v-on-style";
const FIXED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/formatter-on-style-policy-6098/Fixed.vue.txt"
));
const REASON: &str = "normalizeDirectiveShorthands=true conflicts with active vue/v-on-style=longform; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand event style";

fn configure(server: &MaestroServer, root: &Path, value: Value) {
    let source = vize_l0::cstr!("export default {{ vue: {value} }};\n");
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

fn requests(server: &MaestroServer, uri: &Url) -> [Option<Vec<TextEdit>>; 3] {
    let mut range = range_params(uri.clone());
    range.range = Range::new(Position::new(0, 0), Position::new(3, 0));
    let mut on_type = on_type_params(uri.clone());
    on_type.text_document_position.position = Position::new(1, 2);
    on_type.ch = "\n".into();
    [
        futures::executor::block_on(server.formatting(formatting_params(uri.clone()))).unwrap(),
        futures::executor::block_on(server.range_formatting(range)).unwrap(),
        futures::executor::block_on(server.on_type_formatting(on_type)).unwrap(),
    ]
}

thread_local! {
    static LOGS: RefCell<Vec<(String, String)>> = const { RefCell::new(Vec::new()) };
}
struct CapturedWarnings;
struct Message(String);
impl tracing::field::Visit for Message {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        assert_eq!(field.name(), "message");
        self.0 = cstr!("{value:?}");
    }
}
impl<S: tracing::Subscriber> Layer<S> for CapturedWarnings {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _context: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut message = Message(String::new());
        event.record(&mut message);
        LOGS.with_borrow_mut(|rows| rows.push((cstr!("{}", event.metadata().level()), message.0)));
    }
}

#[test]
fn all_three_requests_warn_refuse_without_writes_and_false_restores_fixed_point() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    let file = root.join("Events.vue");
    std::fs::write(&file, FIXED).unwrap();
    let uri = Url::from_file_path(&file).unwrap();
    let service = server_with(&uri, FIXED, "vue");
    let server = service.inner();
    for normalize in [None, Some(true)] {
        configure(server, root, policy(normalize));
        if normalize == Some(true) {
            assert_eq!(
                std::fs::read(root.join("vite.config.mjs")).unwrap(),
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/_fixtures/differential/lsp/formatter-on-style-policy-6098/vite.config.mjs.txt"
                ))
            );
        }
        let config = std::fs::read(root.join("vite.config.mjs")).unwrap();
        LOGS.with_borrow_mut(Vec::clear);
        let subscriber = tracing_subscriber::registry()
            .with(CapturedWarnings.with_filter(tracing_subscriber::filter::LevelFilter::WARN));
        let result = tracing::subscriber::with_default(subscriber, || requests(server, &uri));
        assert_eq!(result, [None, None, None]);
        let expected = (
            String::from("WARN"),
            cstr!(
                "Refusing formatter request for {}: {REASON}",
                file.display()
            ),
        );
        LOGS.with_borrow(|rows| assert_eq!(rows, &vec![expected; 3]));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), FIXED);
        assert_eq!(std::fs::read(root.join("vite.config.mjs")).unwrap(), config);
        assert_eq!(
            server.state.documents.text_snapshot(&uri).unwrap().as_str(),
            FIXED
        );
    }

    configure(server, root, policy(Some(false)));
    let config = std::fs::read(root.join("vite.config.mjs")).unwrap();
    let result = requests(server, &uri);
    let options = vize_glyph::FormatOptions {
        normalize_directive_shorthands: false,
        ..Default::default()
    };
    let formatted = vize_glyph::format_sfc(FIXED, &options).unwrap().code;
    let full = result[0].as_ref().unwrap();
    assert_eq!(full.len(), usize::from(formatted.as_str() != FIXED));
    assert_eq!(
        full.first().map(|edit| edit.new_text.as_str()),
        if formatted.as_str() == FIXED {
            None
        } else {
            Some(formatted.as_str())
        }
    );
    assert!(result[1].is_some());
    server
        .state
        .documents
        .open(uri.clone(), formatted.to_string(), 2, "vue".into());
    assert_eq!(
        requests(server, &uri),
        [Some(Vec::new()), Some(Vec::new()), Some(Vec::new())]
    );
    assert_eq!(std::fs::read_to_string(&file).unwrap(), FIXED);
    assert_eq!(std::fs::read(root.join("vite.config.mjs")).unwrap(), config);
    assert_eq!(
        server.state.documents.text_snapshot(&uri).unwrap().as_str(),
        formatted.as_str()
    );
}

#[test]
fn event_requests_follow_scopes_order_negation_ignore_and_actual_editor_activation() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    let uri = Url::from_file_path(root.join("Root.vue")).unwrap();
    let service = server_with(&uri, FIXED, "vue");
    let server = service.inner();
    configure(
        server,
        root,
        json!({
            "ignores":["ignored/**"],
            "linter":{"preset":"incremental","rules":{RULE:"off"},"ruleOptions":{RULE:"longform"}},
            "entries":[
                {"files":["src/**","!src/Keep.vue"],"linter":{"rules":{RULE:"error"}}},
                {"files":["src/final/**"],"linter":{"rules":{RULE:"off"}}},
                {"basePath":"packages/ui","files":["**/*.vue"],"linter":{"rules":{RULE:"error"}}},
                {"files":["src/short/**"],"linter":{"ruleOptions":{RULE:"shorthand"}}}
            ]
        }),
    );
    for (name, active) in [
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
        std::fs::write(&file, FIXED).unwrap();
        let uri = Url::from_file_path(&file).unwrap();
        server
            .state
            .documents
            .open(uri.clone(), FIXED.to_owned(), 1, "vue".into());
        let result = requests(server, &uri);
        assert_eq!(result.iter().all(Option::is_none), active, "{name}");
        if !active {
            assert!(result[0].is_some());
            assert!(result[1].is_some());
        }
        assert_eq!(std::fs::read_to_string(&file).unwrap(), FIXED);
        assert_eq!(
            server.state.documents.text_snapshot(&uri).unwrap().as_str(),
            FIXED
        );
    }
    // Actual diagnostics use the editor's constructor, not the CLI category pass.
    for (linter, active) in [
        (
            json!({"preset":"incremental","ruleOptions":{RULE:"longform"}}),
            true,
        ),
        (
            json!({"preset":"incremental","categories":{"style":"off"},"ruleOptions":{RULE:"longform"}}),
            true,
        ),
        (
            json!({"preset":"opinionated","categories":{"style":"off"},"ruleOptions":{RULE:"longform"}}),
            true,
        ),
        (
            json!({"preset":"incremental","rules":{RULE:"off"},"ruleOptions":{RULE:"longform"}}),
            false,
        ),
        (
            json!({"preset":"incremental","rules":{"vue/no-v-html":"error"},"ruleOptions":{RULE:"longform"}}),
            false,
        ),
        (
            json!({"preset":"incremental","ruleOptions":{RULE:"longform","script/no-restricted-members":{"members":[{"object":"window","property":"localStorage"}]}}}),
            false,
        ),
        (
            json!({"preset":"incremental","ruleOptions":{RULE:"longform","musea/prefer-design-tokens":{"tokens":[{"path":"color.primary","value":"#3b82f6"}]}}}),
            false,
        ),
    ] {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/formatter-on-style-policy-6098/Original.vue.txt"
        ));
        let file = root.join("Events.vue");
        std::fs::write(&file, source).unwrap();
        let uri = Url::from_file_path(&file).unwrap();
        server
            .state
            .documents
            .open(uri.clone(), source.to_owned(), 1, "vue".into());
        configure(
            server,
            root,
            json!({"languageServer":{"lint":true},"linter":linter}),
        );
        server.state.load_lsp_config(root);
        let diagnostics = crate::ide::DiagnosticService::collect_lint_only(&server.state, &uri);
        let count = diagnostics
            .iter()
            .filter(|row| row.code == Some(NumberOrString::String(RULE.into())))
            .count();
        assert_eq!(count, usize::from(active), "{linter}: {diagnostics:?}");
        let result = requests(server, &uri);
        assert_eq!(result.iter().all(Option::is_none), active, "{linter}");
        if !active {
            assert!(result[0].is_some());
            assert!(result[1].is_some());
        }
        assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
        assert_eq!(
            server.state.documents.text_snapshot(&uri).unwrap().as_str(),
            source
        );
    }
}
