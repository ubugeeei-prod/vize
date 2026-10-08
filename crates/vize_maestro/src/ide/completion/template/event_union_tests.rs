#![expect(clippy::disallowed_methods, reason = "tests use LSP strings")]

use tower_lsp::lsp_types::{
    Command, CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, Range, TextEdit,
};

use super::append_component_surface;

fn event() -> CompletionItem {
    CompletionItem {
        label: "@click".to_string(),
        kind: Some(CompletionItemKind::EVENT),
        insert_text: Some("@click=\"$1\"".to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        ..Default::default()
    }
}

#[test]
fn declared_event_keeps_its_entire_payload_and_unrelated_candidate_order() {
    let fallback = event();
    let mut declared = fallback.clone();
    declared.detail = Some("event: [value: boolean]".to_string());
    declared.documentation = Some(Documentation::String(
        "Authored event documentation".to_string(),
    ));
    declared.sort_text = Some("00-event-click".to_string());
    declared.data = Some(serde_json::json!({ "source": "component" }));
    let unrelated = CompletionItem {
        label: "@focus".to_string(),
        ..event()
    };
    let mut items = vec![fallback, unrelated.clone()];
    append_component_surface(&mut items, vec![declared.clone()]);
    assert_eq!(items, vec![unrelated, declared]);
}

#[test]
fn same_labels_with_distinct_acceptance_actions_remain_available() {
    let declared = event();
    let mut distinct_kind = event();
    distinct_kind.kind = Some(CompletionItemKind::PROPERTY);
    let mut distinct_text = event();
    distinct_text.insert_text = Some("@click=\"handler\"".to_string());
    let mut distinct_format = event();
    distinct_format.insert_text_format = Some(InsertTextFormat::PLAIN_TEXT);
    let mut distinct_command = event();
    distinct_command.command = Some(Command {
        title: "Open event".to_string(),
        command: "vue.openEvent".to_string(),
        arguments: None,
    });
    let mut distinct_import = event();
    distinct_import.additional_text_edits = Some(vec![TextEdit {
        range: Range::default(),
        new_text: "import handler from './handler';\n".to_string(),
    }]);
    let mut modifier = event();
    modifier.label = "@click.stop".to_string();
    modifier.insert_text = Some("@click.stop=\"$1\"".to_string());
    let mut items = vec![
        distinct_kind,
        distinct_text,
        distinct_format,
        distinct_command,
        distinct_import,
        modifier,
    ];
    let mut expected = items.clone();
    expected.push(declared.clone());
    append_component_surface(&mut items, vec![declared]);
    assert_eq!(items, expected);
}
