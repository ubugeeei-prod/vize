//! Read-only formatter requests must expose conflicting lint policy explicitly.

use super::*;
use serde_json::{Value, json};
use std::{path::Path, sync::mpsc};
use tower_lsp::lsp_types::NumberOrString;
use tracing_subscriber::{Layer, layer::SubscriberExt};
use vize_l0::{String as CompactString, cstr};

const RULE: &str = "vue/v-bind-style";
const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/cases.json"
));

fn original_fixed() -> &'static str {
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
    pinned
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

type CapturedEvent = (CompactString, Vec<(CompactString, CompactString)>);

struct CapturedWarnings {
    events: mpsc::Sender<CapturedEvent>,
}
struct Fields(Vec<(CompactString, CompactString)>);

impl tracing::field::Visit for Fields {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.push((field.name().into(), cstr!("{value:?}")));
    }
}

impl<S: tracing::Subscriber> Layer<S> for CapturedWarnings {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _context: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Same admission as `LevelFilter::WARN`: keep WARN and more severe.
        if event.metadata().level() > &tracing::Level::WARN {
            return;
        }
        let mut fields = Fields(Vec::new());
        event.record(&mut fields);
        self.events
            .send((cstr!("{}", event.metadata().level()), fields.0))
            .unwrap();
    }
}

/// Rebuild callsite interest on a thread with no subscriber.
///
/// With one registered dispatcher, that rebuild stores `Interest::never` for
/// every callsite. The three refusal warnings must still be captured.
fn rebuild_interest_from_unsubscribed_thread() {
    let (done_tx, done_rx) = mpsc::channel();
    std::thread::spawn(move || {
        tracing::callsite::rebuild_interest_cache();
        let _ = done_tx.send(());
    });
    done_rx.recv().unwrap();
}

#[test]
fn every_handler_logs_the_conflict_returns_null_and_keeps_authored_bytes() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    let file = root.join("VBindProp.vue");
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp/formatter-warning-capture-11815/Fixed.vue.txt"
    ));
    assert_eq!(source, original_fixed());
    std::fs::write(&file, source).unwrap();
    let uri = Url::from_file_path(&file).unwrap();
    let service = server_with(&uri, source, "vue");
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
            assert_eq!(
                std::fs::read(root.join("vite.config.mjs")).unwrap(),
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/_fixtures/differential/lsp/formatter-warning-capture-11815/vite.config.mjs.txt"
                ))
            );
        }
        let config_before = std::fs::read(root.join("vite.config.mjs")).unwrap();
        let (events, captured) = mpsc::channel();
        let subscriber = tracing_subscriber::registry().with(CapturedWarnings { events });
        let result = tracing::subscriber::with_default(subscriber, || {
            rebuild_interest_from_unsubscribed_thread();
            requests(server, &uri)
        });
        // Each event is sent synchronously before its formatter request returns.
        let logs = captured.try_iter().collect::<Vec<_>>();
        println!("bind-style formatter capture: {normalize:?} {result:?} {logs:?}");
        assert_eq!(result, [None, None, None]);
        let responses: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/lsp/formatter-warning-capture-11815/response.expected.json"
        )))
        .unwrap();
        assert_eq!(
            serde_json::to_value(&result).unwrap(),
            Value::Array(
                responses
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| row["result"].clone())
                    .collect()
            )
        );
        let expected = (
            CompactString::from("WARN"),
            vec![(
                CompactString::from("message"),
                cstr!(
                    "Refusing formatter request for {}: normalizeDirectiveShorthands=true conflicts with active vue/v-bind-style=longform; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand binding style",
                    file.display()
                ),
            )],
        );
        assert_eq!(logs, vec![expected; 3]);
        let warnings: Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/lsp/formatter-warning-capture-11815/warnings.expected.json"
        )))
        .unwrap();
        let file_name = cstr!("{}", file.display());
        let expected_logs = warnings
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                let fields = row["fields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|field| {
                        (
                            CompactString::from(field[0].as_str().unwrap()),
                            CompactString::from(
                                field[1]
                                    .as_str()
                                    .unwrap()
                                    .replace("${file}", file_name.as_str()),
                            ),
                        )
                    })
                    .collect();
                (CompactString::from(row["level"].as_str().unwrap()), fields)
            })
            .collect::<Vec<CapturedEvent>>();
        assert_eq!(logs, expected_logs);
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
    let formatted = vize_glyph::format_sfc(source, &options).unwrap().code;
    let full = result[0].as_ref().unwrap();
    assert_eq!(full.len(), 1);
    assert_eq!(full[0].new_text, formatted.as_str());
    assert_eq!(
        formatted.as_str(),
        "<template>\n  <a v-bind:title.prop=\"t\"></a>\n</template>\n"
    );
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
    let service = server_with(&uri, source, "vue");
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
        std::fs::write(&file, source).unwrap();
        let uri = Url::from_file_path(&file).unwrap();
        server
            .state
            .documents
            .open(uri.clone(), source.to_owned(), 1, "vue".into());
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

#[test]
fn editor_policy_matches_actual_lint_option_registration_and_category_handling() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    let file = root.join("VBindProp.vue");
    // The retained range/on-type requests address the original single line.
    let source = "<template><a :title.prop=\"t\"></a></template>\n";
    std::fs::write(&file, source).unwrap();
    let uri = Url::from_file_path(&file).unwrap();
    let service = server_with(&uri, source, "vue");
    let server = service.inner();
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
        configure(
            server,
            root,
            json!({"languageServer":{"lint":true},"linter":linter}),
        );
        server.state.load_lsp_config(root);
        let diagnostics = crate::ide::DiagnosticService::collect_lint_only(&server.state, &uri);
        let binding_count = diagnostics
            .iter()
            .filter(|row| row.code == Some(NumberOrString::String(RULE.into())))
            .count();
        assert_eq!(binding_count, usize::from(active), "{linter}");
        let results = requests(server, &uri);
        assert_eq!(results.iter().all(Option::is_none), active, "{linter}");
        if !active {
            assert!(results.iter().all(Option::is_some), "{linter}");
        }
        assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
        assert_eq!(
            server.state.documents.text_snapshot(&uri).unwrap().as_str(),
            source
        );
    }
}
