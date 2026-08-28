//! Options API `this.*` diagnostics anchor on authored bytes, not generated ones.
//!
//! The typed-instance bridge re-emits every `methods`/`computed` body inside
//! setup scope with a `this: __VizeThis` receiver. Those copies used to carry
//! no source mapping, so `CompositeSourceMap` fell back to reporting the
//! *generated* offset as if it were authored: a finding in a script block that
//! sits below a long template landed hundreds of lines away, inside whatever
//! block happened to occupy that offset.
//!
//! Because the anchor was a raw generated offset, it also moved whenever the
//! preamble changed size — #3582 shifted the pinned vue2-elm anchors by exactly
//! the number of bytes it added. This test pins the authored line *and* column
//! of the complete diagnostic so neither relapse can pass.

use std::path::Path;

use super::super::{
    BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};
use crate::batch::TypeChecker;
use vize_s0::{String, cstr};

/// A long template above the script block, so an authored offset and the
/// matching generated offset can never coincide by accident.
const APP_VUE: &str = r#"<template>
  <ul class="list">
    <li class="row">item 1</li>
    <li class="row">item 2</li>
    <li class="row">item 3</li>
    <li class="row">item 4</li>
    <li class="row">item 5</li>
    <li class="row">item 6</li>
    <li class="row">item 7</li>
    <li class="row">item 8</li>
    <li class="row">item 9</li>
    <li class="row">item 10</li>
    <li class="row">item 11</li>
    <li class="row">item 12</li>
    <li class="row">item 13</li>
    <li class="row">item 14</li>
    <li class="row">item 15</li>
    <li class="row">item 16</li>
    <li class="row">item 17</li>
    <li class="row">item 18</li>
    <li class="row">item 19</li>
    <li class="row">item 20</li>
  </ul>
</template>

<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
  methods: {
    bump() {
      this.count = this.missingHelper;
    },
  },
};
</script>
"#;

#[test]
fn options_api_method_diagnostics_anchor_on_the_authored_property() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "options-api-bridge-authored-anchor",
        &[("src/App.vue", APP_VUE)],
    );
    if !project_root.join("node_modules/vue/dist").exists() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }

    let snapshot = legacy_vue2_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);

    // `this.missingHelper` is authored at line 33, column 25 — inside the
    // `<script>` block that starts on line 26, never in the template above it.
    assert_eq!(
        snapshot,
        vec![(
            "src/App.vue".into(),
            Some(2339),
            "33:25:error Property 'missingHelper' does not exist on type '__VizeThis'.".into(),
        )]
    );
}

/// [`super::super::snapshot_project_diagnostics`] with the Vue 2 dialect on,
/// which is what turns the Options API typed-instance bridge on.
///
/// The caller already gated on the two legitimate skips (no `tsgo` binary, no
/// installed `vue`), so a checker error past that point is a real defect and
/// must fail the test rather than collapse into a silent pass.
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
