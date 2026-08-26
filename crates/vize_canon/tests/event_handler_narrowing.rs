//! Inline event-handler assignments are runtime callbacks: they must not
//! narrow template-scope refs for sibling bindings (issue #4962).
//!
//! `vue-tsc` 3.3.4 / `vue` 3.6.0-beta.10 reports nothing on these fixtures,
//! so any TS2367 here is a vize-only false positive.

#[path = "support/script_block_project.rs"]
mod project;

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ modelValue?: boolean; isOpenedDialogs?: boolean }>();
defineEmits<{ "bulk-create-contents": []; "edit-contents:csv": [] }>();
</script>
<template><div /></template>
"#;

const PARENT: &str = r#"<script setup lang="ts">
import { ref } from "vue";
import Child from "./child.vue";

const DIALOG_STATE = { CLOSED: "closed", OPENED_A: "a", OPENED_BULK: "bulk" } as const;
type DialogState = (typeof DIALOG_STATE)[keyof typeof DIALOG_STATE];
const dialogState = ref<DialogState>(DIALOG_STATE.CLOSED);
</script>

<template>
  <div>
    <Child
      :is-opened-dialogs="dialogState !== DIALOG_STATE.CLOSED"
      @edit-contents:csv="dialogState = DIALOG_STATE.OPENED_A"
      @bulk-create-contents="dialogState = DIALOG_STATE.OPENED_BULK"
    />
    <Child :model-value="dialogState === DIALOG_STATE.OPENED_A" />
  </div>
</template>
"#;

/// The issue #4962 repro: assignments inside component `@event` handlers must
/// not narrow `dialogState` for the binding written before them on the same
/// element, nor for the later sibling `<Child>` binding.
#[test]
fn emit_handler_assignments_do_not_narrow_sibling_bindings() {
    let rows = project::check(&[("src/parent.vue", PARENT), ("src/child.vue", CHILD)]);
    let ts2367: Vec<&String> = rows.iter().filter(|row| row.contains("TS2367")).collect();
    assert!(
        ts2367.is_empty(),
        "handler assignments must not leak narrowing into sibling bindings (issue #4962): {ts2367:#?}"
    );
}

/// The same leak through a native DOM handler: `@click="state = 'b'"` on a
/// plain element must not narrow the union for sibling bindings either.
#[test]
fn native_handler_assignments_do_not_narrow_sibling_bindings() {
    let source = r#"<script setup lang="ts">
import { ref } from "vue";
const state = ref<"a" | "b" | "closed">("closed");
</script>
<template>
  <div>
    <button :disabled="state !== 'closed'" @click="state = 'b'">x</button>
    <span v-if="state === 'a'">a</span>
  </div>
</template>
"#;
    let rows = project::check(&[("src/App.vue", source)]);
    let ts2367: Vec<&String> = rows.iter().filter(|row| row.contains("TS2367")).collect();
    assert!(
        ts2367.is_empty(),
        "native handler assignments must not leak narrowing into sibling bindings: {ts2367:#?}"
    );
}

/// Handler bodies must still be checked: an assignment of a value outside the
/// union keeps its TS2322 inside the handler scope.
#[test]
fn handler_bodies_are_still_type_checked() {
    let parent = r#"<script setup lang="ts">
import { ref } from "vue";
import Child from "./child.vue";
const state = ref<"a" | "b">("a");
</script>
<template>
  <Child @bulk-create-contents="state = 'nope'" />
</template>
"#;
    let rows = project::check(&[("src/parent.vue", parent), ("src/child.vue", CHILD)]);
    let parent_rows: Vec<&str> = rows
        .iter()
        .filter(|row| row.starts_with("src/parent.vue"))
        .map(|row| row.as_str())
        .collect();
    assert_eq!(
        parent_rows,
        [
            "src/parent.vue(7,33): error TS2322: Type '\"nope\"' is not assignable to type '\"a\" | \"b\"'."
        ],
        "an invalid assignment inside a handler must still be reported with vue-tsc's exact row"
    );
}
