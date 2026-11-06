//! A render-time copy of a guarded component listener must not own nested
//! callback parameter diagnostics; the typed event scope is the owner (#11977).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn guarded_component_listener_keeps_nested_callback_context() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "guarded-listener-nested-callback-11977",
        &[
            (
                "src/Child.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/guarded-listener-nested-callback-11977/Child.vue.txt"
                ),
            ),
            (
                "src/Parent.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/guarded-listener-nested-callback-11977/Parent.vue.txt"
                ),
            ),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(actual, Some(Vec::new()));
}

#[test]
fn nested_callback_body_diagnostic_has_one_typed_owner() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let parent = include_str!(
        "../../../../../../../tests/_fixtures/differential/typechecker/guarded-listener-nested-callback-11977/Parent.vue.txt"
    )
    .replace("x.id", "x.missing");
    let project = super::real_vue::create_project_case(
        "guarded-listener-nested-callback-error-11977",
        &[
            (
                "src/Child.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/guarded-listener-nested-callback-11977/Child.vue.txt"
                ),
            ),
            ("src/Parent.vue", parent.as_str()),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(
        actual,
        Some(vec![(
            vize_l0::String::from("src/Parent.vue"),
            Some(2339),
            vize_l0::String::from(
                "11:45:error Property 'missing' does not exist on type '{ id: string; }'.",
            ),
        )]),
    );
}

#[test]
fn multiline_guarded_listener_keeps_nested_callback_context() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "multiline-guarded-listener-nested-callback-11977",
        &[
            (
                "src/Child.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/guarded-listener-nested-callback-11977/Child.vue.txt"
                ),
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import { ref } from 'vue';
import Child from './Child.vue';

const ready = ref(true);
const ids = ref<string[]>([]);
</script>

<template>
  <div v-if="ready">
    <Child
      @change="
        v => ids = v.map(
          x => x.id
        )
      "
    />
  </div>
</template>
"#,
            ),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(actual, Some(Vec::new()));
}
