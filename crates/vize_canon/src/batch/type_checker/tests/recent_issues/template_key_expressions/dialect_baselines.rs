//! The key shapes #4149 covers that are not the fragment fix itself: the
//! component key the baseline checks and vize does not, and the Vue 2 dialect,
//! which resolves no prop through Vue 3's element table at all.

use std::path::Path;

use super::super::super::{
    BatchTypeChecker, relative_path, resolve_test_tsgo_binary, snapshot_project_diagnostics,
};
use super::create_key_project;
use crate::batch::TypeChecker;
use vize_s0::{String, cstr};

/// A component's `:key` is a reserved prop, kept out of the child's own props
/// and out of its event inference, and vize checks it against nothing.
///
/// This is a *baseline-only* divergence that predates #4149 and is untouched by
/// it: `vue-tsc 3.3.4` checks a component key against `ReservedProps` and
/// reports both rows below, so vize is silent where the baseline speaks. It is
/// pinned here rather than fixed, because reporting it needs a reserved-prop
/// contract the component prop path does not have yet — a new diagnostic, not
/// the removal of a false one. It carries no
/// `compat-documented-differences.json` entry: that ledger cancels observed
/// corpus rows by project, file and span, and no matrix run has produced this
/// one. Whoever adds the contract should delete this test, not adjust it.
///
/// ```text
/// src/Component.vue(8,37): error TS2322: Type 'string | null' is not assignable to type 'PropertyKey | undefined'.
///   Type 'null' is not assignable to type 'PropertyKey | undefined'.
/// src/Component.vue(9,34): error TS2322: Type '{ id: number; }' is not assignable to type 'PropertyKey | undefined'.
/// ```
#[test]
fn component_keys_stay_outside_props_and_events() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_key_project(
        "component-key-reserved-prop-contract",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ label: string }>()
</script>

<template>
  <span>{{ label }}</span>
</template>
"#,
            ),
            (
                "src/Component.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
const buttons: { text: string | null }[] = []
const tabs: { id: number }[] = []
</script>

<template>
  <Child v-for="button in buttons" :key="button.text" label="a" />
  <Child v-for="option in tabs" :key="option" label="b" />
</template>
"#,
            ),
        ],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let snapshot = snapshot.expect("type-check the component key project");

    assert_eq!(snapshot, vec![]);
}

/// The Vue 2 dialect never resolves a prop through Vue 3's element table, so the
/// same authored keys carry no contract there either — and stay ordinary
/// expressions, which is what keeps the invalid member reportable.
///
/// `vue-tsc` with `vueCompilerOptions.target: 2.7` is not part of this
/// workspace's oracle set; the row below is the shape the Vue 3 baseline
/// reports for the identical authored member access.
#[test]
fn vue2_template_keys_carry_no_element_contract() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_key_project(
        "vue2-template-key-contract",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
const items: { id: number }[] = []
</script>

<template>
  <div>
    <template v-for="item in items" :key="item.id">
      <span>{{ item.id }}</span>
    </template>
    <template v-for="item in items" :key="item.missing">
      <span>{{ item.id }}</span>
    </template>
  </div>
</template>
"#,
        )],
    );

    let snapshot = legacy_vue2_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    assert_eq!(
        snapshot,
        vec![(
            String::from("src/App.vue"),
            Some(2339),
            String::from(
                "10:48:error Property 'missing' does not exist on type '{ id: number; }'."
            ),
        )]
    );
}

/// [`snapshot_project_diagnostics`] with the Vue 2 dialect on.
///
/// The caller already gated on the only legitimate skip (no `tsgo` binary), so a
/// checker error past that point is a real defect and must fail the test rather
/// than collapse into a silent pass.
fn legacy_vue2_diagnostics(project_root: &Path) -> Vec<(String, Option<u32>, String)> {
    let mut checker = BatchTypeChecker::new(project_root).expect("batch type checker construction");
    checker.enable_legacy_vue2();
    checker.scan_project().expect("project scan");
    let result = checker.check_project().expect("project check");

    let mut snapshot: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
                    "{}:{}:{} {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    match diagnostic.severity {
                        1 => "error",
                        2 => "warning",
                        3 => "info",
                        _ => "hint",
                    },
                    diagnostic.message
                ),
            )
        })
        .collect();
    snapshot.sort();
    snapshot
}
