use super::RenameService;

#[cfg(feature = "native")]
#[path = "corsa_options_api_tests.rs"]
mod corsa_options_api_tests;

/// Renaming a local binding must not rewrite a same-named prop
/// *attribute name* on a component usage — that token belongs to the
/// child's own prop symbol, and rewriting it silently breaks the call
/// site (#3892). Building rename from the references provider makes the
/// invariant structural: every edit is a reference (or the declaration).
#[test]
fn rename_leaves_same_named_prop_attribute_names_alone() {
    let content = r#"<script setup lang="ts">
import Child from './Child.vue'
const count = 1
</script>

<template>
  <Child :count="count" />
</template>
"#;
    let uri = tower_lsp::lsp_types::Url::parse("file:///ws/Parent.vue").unwrap();
    let state = crate::server::ServerState::new();
    state
        .documents
        .open(uri.clone(), content.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, content);
    let offset = content.find("const count").unwrap() + "const ".len();
    let ctx = crate::ide::IdeContext::new(&state, &uri, offset).unwrap();

    let edit = RenameService::rename(&ctx, "tally").unwrap();
    let edits = &edit.changes.unwrap()[&uri];

    let attr_name_line = content[..content.find(":count=").unwrap()]
        .matches('\n')
        .count() as u32;
    for edit in edits {
        let touches_attr_name = edit.range.start.line == attr_name_line
            && edit.range.start.character
                == content
                    .lines()
                    .nth(attr_name_line as usize)
                    .unwrap()
                    .find(":count")
                    .unwrap() as u32
                    + 1;
        assert!(
            !touches_attr_name,
            "the :count attribute name must not be renamed: {edits:?}"
        );
    }

    // The invariant that pins the providers together: every rename edit is
    // one of the reference locations.
    let references = crate::ide::references::ReferencesService::references(&ctx, true).unwrap();
    for edit in edits {
        assert!(
            references
                .iter()
                .any(|reference| reference.range == edit.range),
            "rename edit {edit:?} is not a reference: {references:?}"
        );
    }
    assert!(
        edits.len() >= 2,
        "the declaration and the binding value must still rename: {edits:?}"
    );
}

#[test]
fn test_get_word_at_offset() {
    let content = "const count = ref(0)";
    assert_eq!(
        RenameService::get_word_at_offset(content, 6),
        Some("count".to_string())
    );
    assert_eq!(
        RenameService::get_word_at_offset(content, 14),
        Some("ref".to_string())
    );
    assert_eq!(
        RenameService::get_word_at_offset(content, 11),
        Some("count".to_string())
    );
    assert_eq!(RenameService::get_word_at_offset(content, 12), None);
}

#[test]
fn test_is_valid_identifier() {
    assert!(RenameService::is_valid_identifier("count"));
    assert!(RenameService::is_valid_identifier("_private"));
    assert!(RenameService::is_valid_identifier("$refs"));
    assert!(!RenameService::is_valid_identifier("123abc"));
    assert!(!RenameService::is_valid_identifier(""));
}

#[test]
fn test_is_keyword() {
    assert!(RenameService::is_keyword("const"));
    assert!(RenameService::is_keyword("function"));
    assert!(!RenameService::is_keyword("count"));
}

#[test]
fn test_offset_range_to_lsp_counts_utf16_code_units() {
    let content = "const emoji = \"😀\"; const message = ref(emoji)";
    let start = content.find("message").unwrap();
    let end = start + "message".len();
    let range = RenameService::offset_range_to_lsp(content, start, end);

    assert_eq!(range.start.line, 0);
    assert_eq!(range.start.character, 26);
    assert_eq!(range.end.character, 33);
}
