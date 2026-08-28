//! A plain Options API component declares no macro props, so its `$props` comes
//! off the authored default export's own instance type rather than the macro
//! prop model — but it must stop resolving to `{}` all the same (#4145).
//!
//! Pinned to the installed `vue`, because this `$props` *is* Vue's own instance
//! type: the diagnostic names `VNodeProps & AllowedComponentProps &
//! ComponentCustomProps`, which the test harness's `vue` stub does not declare.

use super::super::super::{
    BatchTypeChecker, TypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
    with_workspace_node_modules_override,
};
use std::path::Path;
use vize_s0::{String, cstr};

const VALID: &str = r#"<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    label: { type: String, default: '' },
    size: { type: Number, required: true },
  },
})
</script>

<template>
  <div>{{ $props.label }}</div>
  <div>{{ $props.size.toFixed(1) }}</div>
</template>
"#;

const INVALID: &str = r#"<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  props: {
    label: { type: String, default: '' },
    size: { type: Number, required: true },
  },
})
</script>

<template>
  <div>{{ $props.nope }}</div>
  <div>{{ $props.size.toUpperCase() }}</div>
</template>
"#;

/// `vue-tsc` reports nothing for the valid component and exactly these two for
/// the invalid one:
///
/// ```text
/// src/OptionsInvalid.vue(13,18): error TS2339: Property 'nope' does not exist on type 'Partial<{ label: string; }> & Omit<{ readonly size: number; readonly label: string; } & VNodeProps & AllowedComponentProps & ComponentCustomProps, "label">'.
/// src/OptionsInvalid.vue(14,23): error TS2339: Property 'toUpperCase' does not exist on type 'number'.
/// ```
///
/// Vize matches both codes and both authored positions; the members of the
/// resolved instance props type print in a different order.
#[test]
fn options_api_props_reach_template_dollar_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist");
    let test_node_modules = workspace_root.join("tests").join("node_modules");
    if !test_node_modules.join("vue/package.json").is_file() {
        return;
    }
    let project_root = with_workspace_node_modules_override(
        Some(
            test_node_modules
                .to_str()
                .expect("test node_modules path should be UTF-8"),
        ),
        || {
            create_project_case(
                "template-instance-props-options-api",
                &[
                    ("src/OptionsValid.vue", VALID),
                    ("src/OptionsInvalid.vue", INVALID),
                ],
            )
        },
    );

    // The Options API surface is opt-in on the checker, exactly as `vize check`
    // turns it on for a project that authors plain `<script>` components.
    let mut checker = BatchTypeChecker::new(&project_root).unwrap();
    checker.enable_options_api();
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    let mut snapshot: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                relative_path(&project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
                    "{}:{}:error {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    diagnostic.message
                ),
            )
        })
        .collect();
    snapshot.sort();
    let _ = std::fs::remove_dir_all(&project_root);

    let props_type = r#"Partial<{ label: string; }> & Omit<{ readonly label: string; readonly size: number; } & VNodeProps & AllowedComponentProps & ComponentCustomProps, "label">"#;
    assert_eq!(
        snapshot,
        vec![
            (
                String::from("src/OptionsInvalid.vue"),
                Some(2339),
                cstr!("13:18:error Property 'nope' does not exist on type '{props_type}'."),
            ),
            (
                String::from("src/OptionsInvalid.vue"),
                Some(2339),
                cstr!("14:23:error Property 'toUpperCase' does not exist on type 'number'."),
            ),
        ],
        "an Options API `$props` resolves the authored instance's props"
    );
}
