//! P4-5a regression fixture: one SFC through both checker entry points.
//!
//! `vize check` (the batch executor) and the Maestro session sync different
//! virtual documents to different Corsa processes, but both hand the finished
//! diagnostics to the one assembly pass. The authored diagnostics must be the
//! same rows — the session-vs-CLI divergence this pass ended.

use super::editor_typecheck_fixture::{
    resolve_test_tsgo_binary, state_for_fixture, write_corsa_config, write_vue_test_package,
};
use super::{DiagnosticService, sources};
use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString, Url};
use vize_canon::BatchTypeCheckerTrait;
use vize_s0::String;

/// The last three script lines are the keyof-indexed assignment Corsa reports
/// as a false-positive `TS2322`; before P4-5a only `vize check` dropped it.
const APP: &str = r#"<script setup lang="ts">
const count: number = "one";
const label = "ready";
type A = { text: string; count: number };
declare const target: A;
declare const key: string;
const value = null as unknown as A[keyof A];
target[key as keyof A] = value;
</script>

<template>
  <p>{{ label }} {{ count }}</p>
  <p>{{ missing }}</p>
  <!-- @vue-expect-error nothing below fails -->
  <span>{{ label }}</span>
</template>
"#;

/// `(line, column, code, severity, message)` of one authored diagnostic.
type Row = (u32, u32, u32, u8, String);

fn expected() -> Vec<Row> {
    let mut rows = vec![
        (
            1,
            6,
            2322,
            1,
            "Type 'string' is not assignable to type 'number'.".into(),
        ),
        (
            12,
            8,
            2339,
            1,
            "Property 'missing' does not exist on the component instance.".into(),
        ),
        (
            13,
            2,
            2578,
            1,
            "Unused '@vue-expect-error' directive.".into(),
        ),
    ];
    rows.sort();
    rows
}

#[test]
fn cli_and_editor_assemble_identical_diagnostics_for_one_sfc() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::TempDir::new().expect("temp project");
    let root = project.path().canonicalize().expect("canonical root");
    write_vue_test_package(&root);
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .expect("tsconfig");
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("src");
    let app = src.join("App.vue");
    std::fs::write(&app, APP).expect("App.vue");

    let mut checker = vize_canon::BatchTypeChecker::with_options_and_corsa_path(
        &root,
        vize_canon::BatchTypeCheckerOptions::default(),
        Some(&corsa_path),
    )
    .expect("batch checker");
    checker.scan_project().expect("scan");
    let mut cli: Vec<Row> = checker
        .check_project()
        .expect("vize check")
        .diagnostics
        .into_iter()
        .filter(|diagnostic| diagnostic.file == app)
        .map(|diagnostic| {
            (
                diagnostic.line,
                diagnostic.column,
                diagnostic.code.expect("checker code"),
                diagnostic.severity,
                diagnostic.message,
            )
        })
        .collect();
    cli.sort();

    write_corsa_config(&root, &corsa_path);
    let uri = Url::from_file_path(&app).expect("file uri");
    let state = state_for_fixture(&root, &uri, APP);
    state.load_workspace_config(&root);
    let mut editor: Vec<Row> =
        crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri))
            .into_iter()
            .filter(|diagnostic| diagnostic.source.as_deref() == Some(sources::TYPE_CHECKER))
            .map(|diagnostic| {
                let code = match diagnostic.code.expect("checker code") {
                    NumberOrString::Number(code) => code as u32,
                    NumberOrString::String(code) => code.trim_start_matches("TS").parse().unwrap(),
                };
                let severity = match diagnostic.severity.expect("severity") {
                    DiagnosticSeverity::ERROR => 1,
                    DiagnosticSeverity::WARNING => 2,
                    DiagnosticSeverity::INFORMATION => 3,
                    _ => 4,
                };
                (
                    diagnostic.range.start.line,
                    diagnostic.range.start.character,
                    code,
                    severity,
                    diagnostic.message.as_str().into(),
                )
            })
            .collect();
    editor.sort();

    assert_eq!(cli, expected(), "vize check");
    assert_eq!(
        editor, cli,
        "the editor must assemble what vize check assembles"
    );
}
