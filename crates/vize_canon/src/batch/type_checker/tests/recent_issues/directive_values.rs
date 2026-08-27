//! Custom directive values are checked against `Directive<El, Value>` (#3445).
//!
//! A custom directive's bound value used to reach vize's undefined-identifier
//! detector and nothing else, so `v-focus="'nope'"` against a
//! `Directive<HTMLElement, number>` was silent where `vue-tsc` reports
//! `TS2322`. Both oracles below are vue-tsc 3.3.4 on byte-identical workspaces.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_s0::String;

/// vue-tsc 3.3.4:
///
/// ```text
/// src/App.vue(7,17): error TS2322: Type 'string' is not assignable to type 'number'.
/// ```
///
/// Column 17 is the start of the authored value `'nope'` — not the directive
/// name, which is where the native prop check anchors. The correct second usage
/// (`v-focus="1"`) is quiet in both tools and must stay that way.
#[test]
fn custom_directive_value_is_checked_against_its_declared_value_type() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "custom-directive-value-check",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
import type { Directive } from 'vue'
const vFocus: Directive<HTMLElement, number> = () => {}
</script>

<template>
  <div v-focus="'nope'" />
  <div v-focus="1" />
</template>
"#,
        )],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(
        snapshot,
        vec![(
            String::from("src/App.vue"),
            Some(2322),
            String::from("7:17:error Type 'string' is not assignable to type 'number'."),
        )]
    );
}

/// The shapes that must stay silent, each of which would be a false positive on
/// correct code across the ecosystem:
///
/// * `v-global` is registered through `app.directive`, a plugin, or an Options
///   API `directives` block, so no `vGlobal` binding exists. Naming it anyway is
///   `TS2304: Cannot find name` — the single highest-volume risk in this change,
///   since global registration is how most Vue apps ship directives.
/// * `vUntyped: Directive` leaves `Value` at its `any` default, so nothing about
///   the bound value is knowable.
/// * `vNotADirective = 42` does not match `Directive<any, infer V>` at all and
///   resolves to `unknown`, which accepts any value.
///
/// vue-tsc 3.3.4 reports nothing on this file.
#[test]
fn unresolvable_and_untyped_directives_stay_silent() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "custom-directive-value-no-false-positive",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
import type { Directive } from 'vue'
const vUntyped: Directive = () => {}
const vNotADirective = 42
</script>

<template>
  <div v-global="'anything'" />
  <div v-untyped="'anything'" />
  <div v-not-a-directive="'anything'" />
</template>
"#,
        )],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(snapshot, vec![]);
}

/// Collecting the directive value is also what makes it reach the checker *at
/// all*, so an ordinary type error inside the expression is now reported —
/// member access included. This is the broader half of the change: before it,
/// `v-tooltip="user.noSuchField"` was unchecked even though vue-tsc reports it.
#[test]
fn directive_value_expression_is_checked_like_any_other_binding() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "custom-directive-value-expression-check",
        &[(
            "src/App.vue",
            r#"<script setup lang="ts">
import type { Directive } from 'vue'
const vTooltip: Directive<HTMLElement, string> = () => {}
const user = { name: 'a' }
</script>

<template>
  <div v-tooltip="user.title" />
</template>
"#,
        )],
    );

    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(
        snapshot,
        vec![(
            String::from("src/App.vue"),
            Some(2339),
            String::from("8:24:error Property 'title' does not exist on type '{ name: string; }'."),
        )]
    );
}
