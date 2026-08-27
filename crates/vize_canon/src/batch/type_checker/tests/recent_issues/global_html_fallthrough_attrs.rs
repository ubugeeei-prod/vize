//! Strict unknown-prop checking accepts real HTML attributes as fallthrough
//! and reports one diagnostic per literal binding (#4966).
//!
//! A closed child — multi-root, so Vize records no fallthrough target — used
//! to reject `id`, `type`, and `accept` while the same check let `aria-*`
//! surfaces through elsewhere: the allowed surface was inconsistent with what
//! Vue actually falls through at runtime. Any attribute some native element
//! declares (plus custom `data-*`) is now part of the strict surface, while a
//! name no element knows (`depressed`) stays a strict finding.
//!
//! A literal binding also used to produce the same `TS2322` twice — once from
//! the per-prop `const` annotation (`Type '123'`), once from the whole-props
//! object elaboration (`Type 'number'`). `vue-tsc` reports the widened form
//! exactly once; so does Vize now.

use super::super::super::{
    create_project_case, create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics, write_test_vue_stub,
};
use crate::batch::runtime_deps::VUE_RUNTIME_DOM_STUB_TYPES;
use vize_s0::cstr;

/// The isolated stub's `NativeElements` is a `Record<string, …>`, whose key
/// union is `string` — a table like that legitimately opens the whole strict
/// surface, so the test pins a concrete `input` row the way the fallthrough
/// suite does.
const NATIVE_INPUT_TYPES: &str = r#"export interface NativeElements {
  input: { id?: string; type?: string; accept?: string; 'aria-label'?: string; 'aria-activedescendant'?: string };
}
"#;

const MULTI_ROOT_WRAPPER: &str = r#"<script setup lang="ts">
defineProps<{ label?: string }>()
</script>

<template>
  <label>{{ label }}</label>
  <input />
</template>
"#;

#[test]
fn issue_4966_strict_surface_accepts_global_html_attrs_on_closed_components() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case_without_node_modules(
        "issue-4966-global-html-attrs",
        &[
            ("src/Wrapper.vue", MULTI_ROOT_WRAPPER),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Wrapper from './Wrapper.vue'
</script>

<template>
  <Wrapper id="password" type="password" accept=".png" />
  <Wrapper aria-label="ok" aria-activedescendant="x" />
  <Wrapper data-test-id="wrapped" />
  <Wrapper depressed />
</template>
"#,
            ),
        ],
    );
    let node_modules = project_root.join("node_modules");
    write_test_vue_stub(&node_modules).expect("write isolated Vue stub");
    let vue_types = VUE_RUNTIME_DOM_STUB_TYPES.replace(
        "export type NativeElements = Record<string, Record<string, unknown>>;",
        NATIVE_INPUT_TYPES,
    );
    std::fs::write(node_modules.join("@vue/runtime-dom/index.d.ts"), vue_types)
        .expect("pin native input attrs");

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    for attr in [
        "id",
        "type",
        "accept",
        "ariaLabel",
        "ariaActivedescendant",
        "dataTestId",
    ] {
        let complaint = cstr!("'\"{attr}\"' does not exist");
        assert!(
            snapshot
                .iter()
                .all(|(_, _, message)| !message.contains(complaint.as_str())),
            "{attr} is a real HTML attribute and must fall through: {snapshot:#?}"
        );
    }
    assert_eq!(snapshot.len(), 1, "{snapshot:#?}");
    let (file, code, message) = &snapshot[0];
    assert_eq!(file.as_str(), "src/App.vue");
    assert_eq!(*code, Some(2353));
    assert!(
        message.contains("'\"depressed\"' does not exist in type"),
        "unknown attrs must stay strict findings: {message}"
    );
}

#[test]
fn issue_4966_literal_binding_reports_one_widened_diagnostic() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "issue-4966-literal-binding-dedup",
        &[
            (
                "src/Toggle.vue",
                r#"<script setup lang="ts">
defineProps<{ modelValue?: boolean }>()
</script>

<template>
  <span>{{ modelValue }}</span>
  <input type="checkbox" />
</template>
"#,
            ),
            (
                "src/App.vue",
                r#"<script setup lang="ts">
import Toggle from './Toggle.vue'
</script>

<template>
  <Toggle :model-value="123" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    // vue-tsc: exactly one TS2322, rendered with the widened literal.
    assert_eq!(
        snapshot.len(),
        1,
        "one diagnostic per binding: {snapshot:#?}"
    );
    let (file, code, message) = &snapshot[0];
    assert_eq!(file.as_str(), "src/App.vue");
    assert_eq!(*code, Some(2322));
    assert!(
        message.ends_with("Type 'number' is not assignable to type 'boolean | undefined'."),
        "the widened rendering is what vue-tsc reports: {message}"
    );
}
