use super::script_lint::script_suppression_edit;
use super::*;
use crate::{
    ide::{DiagnosticService, position_to_offset},
    server::ServerState,
};
use tower_lsp::lsp_types::{CodeActionOrCommand, NumberOrString, Url};

const SOURCE: &str = r#"<script setup lang="ts">
import { ref } from "vue";

const emit = defineEmits<{ close: [] }>();
const { title } = defineProps<{ title: string }>();
const boxRef = ref<HTMLElement | null>(null);
</script>

<template>
  <div ref="boxRef" @click="emit('close')">{{ title }}</div>
</template>
"#;

fn project() -> (tempfile::TempDir, ServerState, Url) {
    let directory = crate::test_support::tempdir().unwrap();
    std::fs::write(
        directory.path().join("vite.config.mjs"),
        r#"export default { vue: {
            "linter": { "preset": "opinionated" },
            "languageServer": { "lint": true, "codeActions": true }
        } };"#,
    )
    .unwrap();
    let state = ServerState::new();
    state.load_lsp_config(directory.path());
    assert_eq!(
        state.get_linter_config().preset.as_deref(),
        Some("opinionated")
    );
    let uri = Url::from_file_path(directory.path().join("App.vue")).unwrap();
    state
        .documents
        .open(uri.clone(), SOURCE.into(), 1, "vue".into());
    (directory, state, uri)
}

#[test]
fn script_rule_diagnostics_offer_attached_eslint_suppressions() {
    let (directory, state, uri) = project();
    let diagnostics = DiagnosticService::collect(&state, &uri);
    assert!(directory.path().exists());

    for (rule, line) in [
        ("script/define-macros-order", 4),
        ("script/prefer-use-template-ref", 5),
    ] {
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == Some(NumberOrString::String(rule.to_owned())))
            .unwrap_or_else(|| panic!("missing {rule}: {diagnostics:?}"));
        let offset = position_to_offset(
            SOURCE,
            diagnostic.range.start.line,
            diagnostic.range.start.character,
        )
        .unwrap();
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let actions = CodeActionService::code_actions_for_diagnostics(
            &ctx,
            diagnostic.range,
            std::slice::from_ref(diagnostic),
        );
        let action = actions
            .iter()
            .find_map(|action| match action {
                CodeActionOrCommand::CodeAction(action)
                    if action.title
                        == format!("Suppress with eslint-disable-next-line ({rule})") =>
                {
                    Some(action)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("missing suppression for {rule}: {actions:?}"));
        assert_eq!(
            action.diagnostics.as_deref(),
            Some(std::slice::from_ref(diagnostic))
        );
        let edit = &action.edit.as_ref().unwrap().changes.as_ref().unwrap()[&uri][0];
        assert_eq!(
            edit.range,
            Range::new(Position::new(line, 0), Position::new(line, 0))
        );
        assert_eq!(
            edit.new_text,
            format!("// eslint-disable-next-line {rule}\n")
        );
    }
}

#[test]
fn template_rule_diagnostics_do_not_get_script_suppressions() {
    let (directory, state, uri) = project();
    let diagnostics = DiagnosticService::collect(&state, &uri);
    assert!(directory.path().exists());
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code
                == Some(NumberOrString::String(
                    "a11y/click-events-have-key-events".into(),
                ))
        })
        .unwrap();
    let offset = position_to_offset(
        SOURCE,
        diagnostic.range.start.line,
        diagnostic.range.start.character,
    )
    .unwrap();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let actions = CodeActionService::code_actions_for_diagnostics(
        &ctx,
        diagnostic.range,
        std::slice::from_ref(diagnostic),
    );
    assert_eq!(actions.len(), 1);
    let CodeActionOrCommand::CodeAction(action) = &actions[0] else {
        panic!("expected a code action: {actions:?}");
    };
    assert_eq!(
        action.title,
        "Suppress with @vize:forget (a11y/click-events-have-key-events)"
    );
}

#[test]
fn script_suppression_edits_preserve_crlf_indentation_and_merge_rules() {
    let source =
        "<script setup>\r\nif (enabled) {\r\n  const value = source;\r\n}\r\n</script>\r\n";
    let block_start = source.find("if (enabled)").unwrap();
    let block_end = source.find("\r\n</script>").unwrap();
    let block_content = &source[block_start..block_end];
    let relative_start = block_content.find("const value").unwrap();

    let edit = script_suppression_edit(
        source,
        block_content,
        block_start,
        relative_start,
        "script/first-rule",
        "\r\n",
    );
    assert_eq!(
        edit.range,
        Range::new(Position::new(2, 0), Position::new(2, 0))
    );
    assert_eq!(
        edit.new_text,
        "  // eslint-disable-next-line script/first-rule\r\n"
    );

    let with_directive = source.replacen(
        "  const value = source;",
        "  // eslint-disable-next-line script/first-rule\r\n  const value = source;",
        1,
    );
    let block_start = with_directive.find("if (enabled)").unwrap();
    let block_end = with_directive.find("\r\n</script>").unwrap();
    let block_content = &with_directive[block_start..block_end];
    let relative_start = block_content.find("const value").unwrap();
    let edit = script_suppression_edit(
        &with_directive,
        block_content,
        block_start,
        relative_start,
        "script/second-rule",
        "\r\n",
    );
    assert_eq!(
        edit.range,
        Range::new(Position::new(2, 0), Position::new(2, 47))
    );
    assert_eq!(
        edit.new_text,
        "  // eslint-disable-next-line script/first-rule, script/second-rule"
    );
}
