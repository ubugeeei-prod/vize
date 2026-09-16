use super::*;
use crate::server::ServerState;
use tower_lsp::lsp_types::Url;

fn offset_to_position(source: &str, offset: usize) -> Position {
    let (line, character) = crate::ide::offset_to_position(source, offset);
    Position::new(line, character)
}

fn edits(action: &CodeActionOrCommand, uri: &Url) -> (String, Vec<TextEdit>) {
    let CodeActionOrCommand::CodeAction(action) = action else {
        panic!("expected an authored code action");
    };
    let edit = action.edit.as_ref().unwrap();
    assert_eq!(edit.document_changes, None);
    assert_eq!(edit.change_annotations, None);
    let changes = edit.changes.as_ref().unwrap();
    assert_eq!(changes.len(), 1);
    (action.title.clone(), changes.get(uri).unwrap().clone())
}

fn check_source(source: &str, newline: &str) {
    let uri = Url::parse("file:///workspace/Inline.vue").unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let gap = source.find("  class=").unwrap();
    let range = Range::new(
        offset_to_position(source, gap),
        offset_to_position(source, gap + 2),
    );
    let ctx = IdeContext::new(&state, &uri, gap + 1).unwrap();
    let fixed = TextEdit {
        range,
        new_text: " ".into(),
    };

    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
    let template = descriptor.template.as_ref().unwrap();
    let relative_start = template.content[..gap - template.loc.start]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    let insert = template.loc.start + relative_start;
    let indent = get_line_indent(&template.content, gap - template.loc.start);
    let suppress = TextEdit {
        range: Range::new(
            offset_to_position(source, insert),
            offset_to_position(source, insert),
        ),
        new_text: vize_s0::cstr!("{indent}<!-- @vize:forget vue/no-multi-spaces -->{newline}").into(),
    };
    let actions = CodeActionService::code_actions(&ctx, range);
    assert_eq!(
        actions
            .iter()
            .map(|action| edits(action, &uri))
            .collect::<Vec<_>>(),
        vec![
            (
                "Fix: Replace multiple spaces with single space".into(),
                vec![fixed.clone()]
            ),
            (
                "Suppress with @vize:forget (vue/no-multi-spaces)".into(),
                vec![suppress.clone()]
            ),
        ]
    );
    let all = CodeActionService::get_all_fixes(&ctx).unwrap();
    assert_eq!(
        all.changes,
        Some(std::collections::HashMap::from([(uri, vec![fixed])]))
    );

    let mut suppressed = source.to_string();
    suppressed.insert_str(insert, &suppress.new_text);
    let parsed = vize_atelier_sfc::parse_sfc(&suppressed, Default::default()).unwrap();
    let result =
        vize_patina::Linter::new().lint_template(&parsed.template.unwrap().content, "Inline.vue");
    assert_eq!(
        result.diagnostics.len(),
        0,
        "suppression must stay inside the template: {result:?}"
    );
}

#[test]
fn inline_template_actions_use_authored_columns() {
    check_source(
        "<template><div title=\"hello\"  class=\"a\">text</div></template>",
        "\n",
    );
}

#[test]
fn inline_template_actions_preserve_utf16_prefixes() {
    check_source(
        "<!-- \u{1f600} --><template><div title=\"\u{1f600}\"  class=\"a\">text</div></template>",
        "\n",
    );
}

#[test]
fn multiline_template_header_actions_use_content_start() {
    check_source(
        "<template\n  ><div title=\"hello\"  class=\"a\">text</div></template>",
        "\n",
    );
}

#[test]
fn multiline_template_actions_preserve_crlf_and_indent() {
    check_source(
        "<script setup>const label = '\u{1f600}'</script>\r\n<template>\r\n\t<div title=\"hello\"  class=\"a\">text</div>\r\n</template>\r\n",
        "\r\n",
    );
}
