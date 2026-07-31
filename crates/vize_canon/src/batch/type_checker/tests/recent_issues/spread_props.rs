//! `v-bind="object"` prop spreads are type-checked (#3444).
//!
//! A spread contributes no `PassedProp`, so a usage that only spreads had no
//! checkable prop, emitted no check, and passed a wrongly typed bag silently.
//! The spread is now folded into the same props object literal the generic
//! inference call already assembles.
//!
//! Oracles below are `vue-tsc@3.3.4` with `vue@3.6.0-beta.10`, run against a
//! byte-identical workspace.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ count: number }>()
</script>

<template><span /></template>
"#;

/// vue-tsc, on the fixture below:
///
/// ```text
/// src/Parent.vue(8,4):  error TS2345: Argument of type '{ count: string; }' is not assignable ...
/// src/Parent.vue(10,4): error TS2345: Argument of type '{}' is not assignable ... Property 'count' is missing.
/// ```
///
/// Line 9 — a bag that satisfies the child — is clean in both tools.
///
/// Column 4 is the `C` of `<Child`, the element name, which is where vue-tsc
/// anchors a whole-props failure. The message text diverges: vize names its own
/// generated props type where vue-tsc names
/// `{ readonly count: number; } & VNodeProps & …`. That is a message-only
/// divergence of the kind the ledger scores separately.
#[test]
fn a_wrongly_typed_spread_is_reported_and_a_correct_one_is_not() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "vbind-spread-props",
        &[
            ("src/Child.vue", CHILD),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
const bag = { count: 'oops' }
const good = { count: 1 }
</script>

<template>
  <Child v-bind="bag" />
  <Child v-bind="good" />
  <Child v-bind="$attrs" />
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

    // `snapshot_project_diagnostics` sorts, so the expectation is sorted too.
    let mut positions: Vec<_> = snapshot
        .iter()
        .map(|(file, code, message)| {
            (
                file.as_str(),
                *code,
                message.split(':').take(2).collect::<Vec<_>>().join(":"),
            )
        })
        .collect();
    positions.sort();
    assert_eq!(
        positions,
        vec![
            ("src/Parent.vue", Some(2345), "10:4".to_string()),
            ("src/Parent.vue", Some(2345), "8:4".to_string()),
        ],
        "positions byte-identical with vue-tsc: {snapshot:?}"
    );
}

/// A spread beside named bindings keeps the named ones on the per-prop path,
/// which owns their anchors, and folds the spread into the same literal so an
/// error *inside* the spread expression still lands on the authored bytes.
#[test]
fn a_spread_beside_named_props_keeps_both_paths() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "vbind-spread-mixed",
        &[
            ("src/Child.vue", CHILD),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
const rest = { extra: 1 }
</script>

<template>
  <Child v-bind="rest" :count="'nope'" />
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

    assert_eq!(
        snapshot.len(),
        1,
        "the named prop is reported once, by the per-prop check: {snapshot:?}"
    );
    assert_eq!(snapshot[0].1, Some(2322));
    assert!(
        snapshot[0].2.starts_with("7:25"),
        "anchored on the `count` attribute name, one byte right of the `:`, got: {}",
        snapshot[0].2
    );
}

/// An undefined name inside the spread expression is reported on the authored
/// bytes, not on the generated literal.
#[test]
fn an_error_inside_the_spread_expression_lands_on_the_source() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "vbind-spread-inner-error",
        &[
            ("src/Child.vue", CHILD),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
const bag = { count: 1 }
</script>

<template>
  <Child v-bind="bag.missing" />
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

    assert!(
        snapshot
            .iter()
            .any(|(_, code, message)| *code == Some(2339) && message.starts_with("7:")),
        "the property error should anchor inside the template: {snapshot:?}"
    );
}

/// A reserved-name prop is a valid bare template binding even though it cannot
/// be emitted as a bare TypeScript identifier. Nuxt UI uses this shape for
/// `v-bind="{ as, ... }"` on its link primitives.
#[test]
fn a_spread_object_rewrites_reserved_template_prop_references() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "vbind-spread-reserved-template-prop",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{ as?: string; value: number; label: string; kind: string; options: object }>()
</script>

<template><span /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
defineProps<{ as?: string; typeof?: string; count: number }>()
</script>

<template>
  <Child v-bind="{
    /* as before */ as /* as after */,
    value: count as /* as assertion */ number,
    label: `${as}`,
    kind: typeof count,
    options: {
      method: { as<T>(value: T) { return `${as}:${value}` } },
      accessor: { get as() { return 'div' }, set as(value) {} },
      generator: { *as<T>() { yield 'div' as T } },
      asyncMethod: { async as<T>() { return 'div' as T } },
      asyncGenerator: { async *as<T>() { yield 'div' as T } }
    }
  }" />
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

    assert_eq!(
        snapshot,
        Vec::new(),
        "reserved template props in spread objects are valid: {snapshot:?}"
    );
}

/// Expanding `{ as }` adds generated bytes before `bag.missing`; its property
/// diagnostic must still land on the exact authored identifier.
#[test]
fn a_spread_rewrite_preserves_the_following_diagnostic_column() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "vbind-spread-reserved-mapping",
        &[
            ("src/Child.vue", CHILD),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
defineProps<{ as?: string }>()
const bag = { count: 1 }
</script>

<template>
  <Child v-bind="{ as, count: bag.missing }" />
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

    let property_errors: Vec<_> = snapshot
        .iter()
        .filter(|(_, code, _)| *code == Some(2339))
        .collect();
    assert_eq!(
        property_errors.len(),
        1,
        "the spread expression must not emit duplicate diagnostics: {snapshot:?}"
    );
    assert!(
        property_errors[0].2.starts_with("8:35:"),
        "the property error should anchor on `missing`: {snapshot:?}"
    );
}
