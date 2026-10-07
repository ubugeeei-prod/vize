use super::*;
use crate::server::ServerState;
use tower_lsp::lsp_types::{Diagnostic, NumberOrString, Url};

const ORIGINAL: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/code-action-diagnostics/App.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/code-action-diagnostics/vize.config.json"
);
const REFERENCE: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/code-action-diagnostics/references.json"
);

fn diagnostics() -> Vec<Diagnostic> {
    let reference: serde_json::Value = serde_json::from_str(REFERENCE).unwrap();
    serde_json::from_value(reference["diagnostics"].clone()).unwrap()
}

fn actions(state: &ServerState, uri: &Url, requested: &[Diagnostic]) -> Vec<CodeActionOrCommand> {
    let range = Range::new(Position::new(1, 2), Position::new(1, 23));
    let ctx = IdeContext::new(state, uri, 13).unwrap();
    CodeActionService::code_actions_for_diagnostics(&ctx, range, requested)
}

fn expected(uri: &Url, diagnostic: &Diagnostic, attach: bool) -> CodeActionOrCommand {
    let Some(NumberOrString::String(rule)) = diagnostic.code.as_ref() else {
        panic!("the authored fixture requires a rule code");
    };
    CodeActionOrCommand::CodeAction(CodeAction {
        title: vize_l0::cstr!("Suppress with @vize:forget ({rule})").into(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: attach.then(|| vec![diagnostic.clone()]),
        edit: Some(WorkspaceEdit {
            changes: Some(std::collections::HashMap::from([(
                uri.clone(),
                vec![TextEdit {
                    range: Range::new(Position::new(1, 0), Position::new(1, 0)),
                    new_text: vize_l0::cstr!("  <!-- @vize:forget {rule} -->\n").into(),
                }],
            )])),
            ..Default::default()
        }),
        is_preferred: Some(false),
        ..Default::default()
    })
}

fn expected_actions(uri: &Url, diagnostic: &Diagnostic, attach: bool) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();
    if diagnostic.code == Some(NumberOrString::String("vue/html-self-closing".into())) {
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Fix: Use self-closing syntax".into(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: attach.then(|| vec![diagnostic.clone()]),
            edit: Some(WorkspaceEdit {
                changes: Some(std::collections::HashMap::from([(
                    uri.clone(),
                    vec![TextEdit {
                        range: Range::new(Position::new(1, 2), Position::new(1, 23)),
                        new_text: "<img src=\"/logo.png\" />".into(),
                    }],
                )])),
                ..Default::default()
            }),
            is_preferred: Some(true),
            ..Default::default()
        }));
    }
    actions.push(expected(uri, diagnostic, attach));
    actions
}

fn expected_all(uri: &Url, diagnostics: &[Diagnostic], attach: bool) -> Vec<CodeActionOrCommand> {
    assert_eq!(diagnostics.len(), 2);
    // The unchanged service collects fixes first, then both suppressions.
    let mut all = expected_actions(uri, &diagnostics[1], attach);
    all.insert(1, expected(uri, &diagnostics[0], attach));
    all
}

fn project() -> (tempfile::TempDir, ServerState, Url) {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("vize.config.json"), CONFIG).unwrap();
    let state = ServerState::new();
    state.load_lsp_config(directory.path());
    let uri = Url::from_file_path(directory.path().join("App.vue")).unwrap();
    state
        .documents
        .open(uri.clone(), ORIGINAL.into(), 1, "vue".into());
    (directory, state, uri)
}

#[test]
fn same_range_rules_use_configured_diagnostic_identity_and_full_payload() {
    let (_directory, state, uri) = project();
    let diagnostics = diagnostics();
    for diagnostic in &diagnostics {
        assert_eq!(
            actions(&state, &uri, std::slice::from_ref(diagnostic)),
            expected_actions(&uri, diagnostic, true)
        );
    }
    let all = expected_all(&uri, &diagnostics, true);
    assert_eq!(actions(&state, &uri, &diagnostics), all);
    let mut reversed = diagnostics.clone();
    reversed.reverse();
    assert_eq!(actions(&state, &uri, &reversed), all);
    let duplicated = [diagnostics.clone(), diagnostics.clone()].concat();
    assert_eq!(actions(&state, &uri, &duplicated), all);
    assert_eq!(
        actions(&state, &uri, &[]),
        expected_all(&uri, &diagnostics, false)
    );
}

#[test]
fn foreign_rule_source_numeric_code_and_stale_range_do_not_select_lint_actions() {
    let (_directory, state, uri) = project();
    let original = diagnostics()[0].clone();
    let mut unrelated = original.clone();
    unrelated.code = Some(NumberOrString::String("vue/no-multi-spaces".into()));
    let mut native = original.clone();
    native.source = Some("vize/types".into());
    native.code = Some(NumberOrString::Number(2322));
    let mut wrong_source = original.clone();
    wrong_source.source = Some("another/linter".into());
    let mut stale = original.clone();
    stale.range.start.character += 1;
    let mut no_code = original.clone();
    no_code.code = None;
    let mut no_source = original.clone();
    no_source.source = None;
    for rejected in [unrelated, native, wrong_source, stale, no_code, no_source] {
        assert_eq!(
            actions(&state, &uri, &[rejected]),
            Vec::<CodeActionOrCommand>::new()
        );
    }
    assert_eq!(
        actions(&state, &uri, std::slice::from_ref(&original)),
        expected_actions(&uri, &original, true)
    );
}

#[test]
fn current_buffer_and_config_refuse_obsolete_diagnostics_without_losing_repaired_actions() {
    let (directory, state, uri) = project();
    let old = diagnostics();
    let reference: serde_json::Value = serde_json::from_str(REFERENCE).unwrap();
    let fixed = reference["fixedSource"].as_str().unwrap();
    state
        .documents
        .open(uri.clone(), fixed.into(), 2, "vue".into());
    assert_eq!(
        actions(&state, &uri, &old),
        Vec::<CodeActionOrCommand>::new()
    );
    state
        .documents
        .open(uri.clone(), ORIGINAL.into(), 3, "vue".into());
    assert_eq!(actions(&state, &uri, &old), expected_all(&uri, &old, true));
    std::fs::write(
        directory.path().join("vize.config.json"),
        "{\"linter\":{\"enabled\":false}}",
    )
    .unwrap();
    state.load_lsp_config(directory.path());
    assert_eq!(
        actions(&state, &uri, &old),
        Vec::<CodeActionOrCommand>::new()
    );
}
