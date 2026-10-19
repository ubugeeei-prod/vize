//! `.value` wrap/unwrap and directory auto-import quick fixes on real SFCs.

use std::fs;

use tower_lsp::lsp_types::{CodeActionOrCommand, Position, Range, TextEdit, Url};

use super::CodeActionService;
use crate::{ide::IdeContext, server::ServerState};

const SOURCE: &str = r#"<script setup lang="ts">
import { ref, computed, reactive } from 'vue'
const count = ref(0)
const doubled = computed(() => count.value * 2)
const state = reactive({ n: 1 })
function bump() {
  count.value++
  console.log(count, doubled.value, state.n)
}
</script>

<template>
  <button @click="bump">{{ count }}</button>
</template>
"#;

/// `(title, edits)` of every action at the cursor `nth` bytes into `needle`.
fn actions_at(
    name: &str,
    source: &str,
    needle: &str,
    nth: usize,
    siblings: &[(&str, &str)],
) -> Vec<(String, Vec<TextEdit>)> {
    let dir = tempfile::tempdir().unwrap();
    for (file, content) in siblings {
        fs::write(dir.path().join(file), content).unwrap();
    }
    let path = dir.path().join(name);
    fs::write(&path, source).unwrap();
    let uri = Url::from_file_path(&path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&uri, source);
    let offset = source.find(needle).unwrap() + nth;
    let (line, character) = crate::ide::offset_to_position(source, offset);
    let position = Position::new(line, character);
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    CodeActionService::code_actions(&ctx, Range::new(position, position))
        .into_iter()
        .map(|action| {
            let CodeActionOrCommand::CodeAction(action) = action else {
                panic!("expected a code action");
            };
            let mut changes = action
                .edit
                .and_then(|edit| edit.changes)
                .unwrap_or_default();
            let edits = changes.remove(&uri).unwrap_or_default();
            (action.title, edits)
        })
        .collect()
}

fn edit(line: u32, start: u32, end: u32, text: &str) -> TextEdit {
    TextEdit {
        range: Range::new(Position::new(line, start), Position::new(line, end)),
        new_text: text.to_owned(),
    }
}

#[test]
fn a_bare_script_ref_offers_wrap_with_value() {
    let actions = actions_at("Counter.vue", SOURCE, "log(count", "log(co".len(), &[]);
    assert_eq!(
        actions,
        vec![(
            "Wrap `count` with `.value`".to_owned(),
            vec![edit(7, 19, 19, ".value")]
        )]
    );
}

#[test]
fn a_computed_value_access_offers_unwrap_only() {
    let actions = actions_at(
        "Counter.vue",
        SOURCE,
        "doubled.value,",
        "doubled.va".len(),
        &[],
    );
    assert_eq!(
        actions,
        vec![(
            "Unwrap `.value` from `doubled`".to_owned(),
            vec![edit(7, 28, 34, "")]
        )]
    );
}

#[test]
fn a_ref_already_followed_by_value_is_not_wrapped_again() {
    let actions = actions_at("Counter.vue", SOURCE, "count.value++", "cou".len(), &[]);
    let titles: Vec<&str> = actions.iter().map(|(title, _)| title.as_str()).collect();
    assert_eq!(titles, Vec::<&str>::new());
}

#[test]
fn a_ref_declaration_name_is_not_wrapped() {
    let actions = actions_at("Counter.vue", SOURCE, "const count", "const co".len(), &[]);
    assert_eq!(actions, Vec::new());
}

#[test]
fn reactive_objects_and_template_refs_get_no_value_fix() {
    assert_eq!(
        actions_at("Counter.vue", SOURCE, "state.n)", "sta".len(), &[]),
        Vec::new()
    );
    assert_eq!(
        actions_at("Counter.vue", SOURCE, "{{ count }}", "{{ co".len(), &[]),
        Vec::new()
    );
}

#[test]
fn an_undeclared_component_name_offers_a_sibling_auto_import() {
    let source = "<script setup lang=\"ts\">\nconst open = true\n</script>\n\n<template>\n  <BaseDialog v-if=\"open\" />\n</template>\n";
    let actions = actions_at(
        "Page.vue",
        source,
        "BaseDialog v-if",
        "Base".len(),
        &[("BaseDialog.vue", "<template><dialog /></template>\n")],
    );
    assert_eq!(
        actions,
        vec![(
            "Auto-import `BaseDialog` from `./BaseDialog.vue`".to_owned(),
            vec![edit(1, 0, 0, "import BaseDialog from './BaseDialog.vue'\n")]
        )]
    );
}
