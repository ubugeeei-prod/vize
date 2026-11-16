//! Plugin lint rules in the editor (#13074): one sidecar per config
//! generation, bounded requests and visible failures.

use std::path::Path;
use std::time::{Duration, Instant};

use tower_lsp::lsp_types::{
    CodeActionOrCommand, Diagnostic, NumberOrString, Position, Range, TextEdit, Url,
};

use crate::ide::{CodeActionService, DiagnosticService, IdeContext};
use crate::server::ServerState;

const BUTTON: &str = "<template>\n  <button :color=\"'red'\">Go</button>\n</template>\n";

/// A config whose plugin rules misbehave for chosen file names. Every
/// evaluation of the config appends one line to `evaluations.log`, and every
/// rule run one line to `runs.log`.
fn project(rules: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let sdk = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../npm/plugin-sdk/v2.js")
        .canonicalize()
        .unwrap();
    let log = dir.path().join("evaluations.log");
    let runs = dir.path().join("runs.log");
    let config = vize_l0::cstr!(
        r#"import {{ appendFileSync }} from "node:fs";
import {{ pathToFileURL }} from "node:url";
appendFileSync({log:?}, "x\n");
const {{ defineVizePlugin }} = await import(pathToFileURL({sdk:?}).href);
const design = defineVizePlugin({{
  apiVersion: 2, stability: "experimental", name: "@acme/design", version: "1.0.0",
  setup(api, options) {{
    api.lint.rule({{
      id: "no-raw-color",
      meta: {{ messages: {{ raw: "Use {{token}} instead of {{value}}." }}, defaultLevel: "warn", fixable: "safe" }},
      visit: ["ui.bind"],
      run(context) {{
        appendFileSync({runs:?}, "x\n");
        if (context.file.endsWith("Throw.vue")) throw new Error("rule-specificreason");
        if (context.file.endsWith("Loop.vue")) for (;;) {{}}
        for (const node of context.nodes) {{
          if (node.name === "color") context.report({{ node, messageId: "raw", data: {{ token: options.token, value: node.value }}, fix: "color" }});
        }}
      }},
    }});
  }},
}});
export default {{ vue: {{ plugins: [design({{ token: "$brand" }})], linter: {{ preset: "essential", rules: {rules} }} }} }};
"#,
        log = log,
        runs = runs,
        sdk = sdk,
    );
    std::fs::write(dir.path().join("vite.config.mjs"), config.as_str()).unwrap();
    dir
}

fn evaluations(dir: &Path) -> usize {
    lines(dir, "evaluations.log")
}

fn lines(dir: &Path, name: &str) -> usize {
    std::fs::read_to_string(dir.join(name))
        .unwrap_or_default()
        .lines()
        .count()
}

fn plugin_diagnostics(state: &ServerState, uri: &Url) -> Vec<Diagnostic> {
    DiagnosticService::collect_lint_only(state, uri)
        .into_iter()
        .filter(|diagnostic| {
            matches!(&diagnostic.code, Some(NumberOrString::String(code)) if code.starts_with("@acme/") || code == "plugin")
        })
        .collect()
}

fn open(state: &ServerState, dir: &Path, name: &str, source: &str, version: i32) -> Url {
    let uri = Url::from_file_path(dir.join(name)).unwrap();
    state
        .documents
        .open(uri.clone(), source.into(), version, "vue".into());
    uri
}

fn range(line: u32, start: u32, end: u32) -> Range {
    Range {
        start: Position {
            line,
            character: start,
        },
        end: Position {
            line,
            character: end,
        },
    }
}

#[test]
fn plugin_rules_lint_editor_documents_through_one_sidecar() {
    let dir = project("{}");
    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    let loaded = evaluations(dir.path());
    assert!(loaded >= 1);

    let uri = open(&state, dir.path(), "Button.vue", BUTTON, 1);
    let found = plugin_diagnostics(&state, &uri);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        found[0].code,
        Some(NumberOrString::String("@acme/design/no-raw-color".into()))
    );
    assert_eq!(found[0].message, "Use $brand instead of 'red'.");
    assert_eq!(found[0].range, range(1, 10, 24));

    // Edits and other documents reuse the running sidecar.
    let moved = vize_l0::cstr!("<!-- 😀 -->\n{BUTTON}");
    open(&state, dir.path(), "Button.vue", &moved, 2);
    let found = plugin_diagnostics(&state, &uri);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].range, range(2, 10, 24));
    let other = open(
        &state,
        dir.path(),
        "Plain.vue",
        "<template><p>ok</p></template>\n",
        1,
    );
    assert!(plugin_diagnostics(&state, &other).is_empty());
    assert_eq!(evaluations(dir.path()), loaded + 1);
}

#[test]
fn rules_that_are_off_start_no_sidecar() {
    let dir = project("{ \"@acme/design/no-raw-color\": \"off\" }");
    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    let loaded = evaluations(dir.path());
    let uri = open(&state, dir.path(), "Button.vue", BUTTON, 1);
    assert!(plugin_diagnostics(&state, &uri).is_empty());
    assert_eq!(evaluations(dir.path()), loaded);
}

#[test]
fn rule_failures_are_visible_and_keep_the_sidecar() {
    let dir = project("{}");
    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    let loaded = evaluations(dir.path());
    let uri = open(&state, dir.path(), "Throw.vue", BUTTON, 1);
    let found = plugin_diagnostics(&state, &uri);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(found[0].code, Some(NumberOrString::String("plugin".into())));
    assert_eq!(found[0].range, range(0, 0, 0));
    assert_eq!(
        found[0].message,
        vize_l0::cstr!(
            "[plugin/rule-failed] @acme/design @acme/design/no-raw-color: {}: rule-specificreason",
            uri.path()
        )
        .as_str()
    );
    let button = open(&state, dir.path(), "Button.vue", BUTTON, 1);
    assert_eq!(plugin_diagnostics(&state, &button).len(), 1);
    assert_eq!(evaluations(dir.path()), loaded + 1);
}

#[test]
fn a_rule_that_never_returns_is_stopped_and_the_next_lint_restarts_it() {
    let dir = project("{}");
    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    let loaded = evaluations(dir.path());
    let looping = open(&state, dir.path(), "Loop.vue", BUTTON, 1);
    let started = Instant::now();
    let found = plugin_diagnostics(&state, &looping);
    let elapsed = started.elapsed();
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(
        found[0].message,
        "[plugin/timeout] plan: plugin lint rules did not answer within 2000 ms"
    );
    assert!(
        elapsed >= Duration::from_secs(2) && elapsed < Duration::from_secs(20),
        "{elapsed:?}"
    );
    let button = open(&state, dir.path(), "Button.vue", BUTTON, 1);
    assert_eq!(plugin_diagnostics(&state, &button).len(), 1);
    assert_eq!(evaluations(dir.path()), loaded + 2);
}

#[test]
fn a_newer_document_version_cancels_the_wait() {
    let dir = project("{}");
    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    let button = open(&state, dir.path(), "Button.vue", BUTTON, 1);
    assert_eq!(plugin_diagnostics(&state, &button).len(), 1);
    let looping = open(&state, dir.path(), "Loop.vue", BUTTON, 1);
    std::thread::scope(|scope| {
        let edit = scope.spawn(|| {
            std::thread::sleep(Duration::from_millis(300));
            open(&state, dir.path(), "Loop.vue", BUTTON, 2);
        });
        let started = Instant::now();
        // The superseded pass publishes nothing of its own.
        assert!(plugin_diagnostics(&state, &looping).is_empty());
        assert!(started.elapsed() < Duration::from_secs(2));
        edit.join().unwrap();
    });
}

#[test]
fn safe_plugin_fixes_are_preferred_quick_fixes_without_another_crossing() {
    let dir = project("{}");
    let state = ServerState::new();
    state.load_workspace_config(dir.path());
    let uri = open(&state, dir.path(), "Button.vue", BUTTON, 1);
    let found = plugin_diagnostics(&state, &uri);
    assert_eq!(found.len(), 1, "{found:#?}");
    assert_eq!(lines(dir.path(), "runs.log"), 1);

    let ctx = IdeContext::new(&state, &uri, BUTTON.find(":color").unwrap()).unwrap();
    let actions =
        CodeActionService::code_actions_for_diagnostics(&ctx, found[0].range, &found[..1]);
    let fixes: Vec<_> = actions
        .iter()
        .filter_map(|action| match action {
            CodeActionOrCommand::CodeAction(action)
                if action.title == "Fix: Apply the @acme/design/no-raw-color fix" =>
            {
                Some(action)
            }
            _ => None,
        })
        .collect();
    assert_eq!(fixes.len(), 1, "{actions:#?}");
    assert_eq!(fixes[0].is_preferred, Some(true));
    assert_eq!(fixes[0].diagnostics.as_deref(), Some(&found[..1]));
    let edits = &fixes[0].edit.as_ref().unwrap().changes.as_ref().unwrap()[&uri];
    assert_eq!(
        edits,
        &vec![TextEdit {
            range: range(1, 10, 24),
            new_text: "color".into(),
        }]
    );
    // The same text reuses the answer the diagnostics already crossed for.
    assert_eq!(lines(dir.path(), "runs.log"), 1);
}
