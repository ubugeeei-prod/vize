//! Go-to-definition from a parent's `#slot` / `v-slot:slot` onto the child's
//! `defineSlots` key.

use std::fs;

use tower_lsp::lsp_types::{Position, Range, Url};

use super::super::DefinitionService;
use super::scalar_location;
use crate::{ide::IdeContext, server::ServerState};

const CHILD: &str = r#"<script setup lang="ts">
defineSlots<{
  default: (props: { item: string }) => any
  'list-item'(props: { index: number }): any
  footer?: () => any
}>()
</script>

<template>
  <slot :item="'x'" />
  <slot name="list-item" :index="0" />
  <slot name="footer" />
</template>
"#;

fn definition_in_child(parent: &str, cursor: &str, offset_in_cursor: usize) -> Option<Range> {
    let dir = tempfile::tempdir().unwrap();
    let child_path = dir.path().join("List.vue");
    let parent_path = dir.path().join("Parent.vue");
    fs::write(&child_path, CHILD).unwrap();
    fs::write(&parent_path, parent).unwrap();

    let uri = Url::from_file_path(&parent_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), parent.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&uri, parent);

    let offset = parent.find(cursor).unwrap() + offset_in_cursor;
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let location = scalar_location(DefinitionService::definition(&ctx)?);
    assert_eq!(
        location.uri.to_file_path().unwrap().canonicalize().unwrap(),
        child_path.canonicalize().unwrap()
    );
    Some(location.range)
}

fn parent_with(slot_template: &str) -> String {
    let mut source = String::from(
        "<script setup lang=\"ts\">\nimport List from './List.vue'\n</script>\n\n<template>\n  <List>\n",
    );
    source.push_str(slot_template);
    source.push_str("\n  </List>\n</template>\n");
    source
}

fn line_range(line: u32, start: u32, len: u32) -> Range {
    Range {
        start: Position {
            line,
            character: start,
        },
        end: Position {
            line,
            character: start + len,
        },
    }
}

#[test]
fn shorthand_slot_name_resolves_to_its_define_slots_key() {
    let parent = parent_with("    <template #default=\"{ item }\">{{ item }}</template>");
    let range = definition_in_child(&parent, "#default", 3);
    assert_eq!(range, Some(line_range(2, 2, "default".len() as u32)));
}

#[test]
fn quoted_method_slot_key_resolves_from_a_kebab_shorthand() {
    let parent = parent_with("    <template #list-item=\"{ index }\">{{ index }}</template>");
    let range = definition_in_child(&parent, "#list-item", 4);
    assert_eq!(range, Some(line_range(3, 3, "list-item".len() as u32)));
}

#[test]
fn a_slot_key_after_an_arrow_function_type_still_resolves() {
    // `=>` in an earlier slot's type must not be read as the `>` that closes
    // `defineSlots<…>`.
    let parent = parent_with("    <template v-slot:footer>Footer</template>");
    let range = definition_in_child(&parent, "v-slot:footer", "v-slot:fo".len());
    assert_eq!(range, Some(line_range(4, 2, "footer".len() as u32)));
}

#[test]
fn a_short_slot_name_is_located_after_the_directive_prefix() {
    // `t` also occurs inside `v-slot`; the cursor on the argument must still
    // resolve.
    let child_slots = CHILD.replace("footer?: () => any", "t: () => any");
    let dir = tempfile::tempdir().unwrap();
    let child_path = dir.path().join("List.vue");
    let parent_path = dir.path().join("Parent.vue");
    fs::write(&child_path, &child_slots).unwrap();
    let parent = parent_with("    <template v-slot:t>T</template>");
    fs::write(&parent_path, &parent).unwrap();

    let uri = Url::from_file_path(&parent_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), parent.clone(), 1, "vue".to_owned());
    state.update_virtual_docs(&uri, &parent);
    let offset = parent.find("v-slot:t").unwrap() + "v-slot:".len();
    let ctx = IdeContext::new(&state, &uri, offset).unwrap();
    let location = scalar_location(DefinitionService::definition(&ctx).unwrap());

    assert_eq!(location.range, line_range(4, 2, 1));
}

#[test]
fn a_slot_missing_from_define_slots_does_not_jump_to_a_lookalike() {
    // `item` appears inside the default slot's props type, not as a key.
    let parent = parent_with("    <template #item>Item</template>");
    let range = definition_in_child(&parent, "#item", 2);
    assert_eq!(range, None);
}
