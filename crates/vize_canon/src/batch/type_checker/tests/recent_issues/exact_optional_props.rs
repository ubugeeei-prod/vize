//! `exactOptionalPropertyTypes` is honored for component props (#3450).
//!
//! Passing an explicitly `undefined` value to an optional child prop is an error
//! under the option and was silent in vize. The per-prop check cannot express
//! it: it extracts `label?: string` to `string | undefined` and assigns to that,
//! which is legal whatever the option says. The distinction between "absent" and
//! "present and `undefined`" only exists when a whole object literal is assigned
//! at once, so the usage's props literal is checked against the child's props
//! type as a whole (`__Child_Props_N`).
//!
//! Every case below is a guard on that check reporting *only* what the per-prop
//! path cannot, because anything else it reports is a second diagnostic for a
//! defect already flagged at a better anchor.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use std::path::Path;
use vize_s0::{String, cstr};

const CHILD: &str = r#"<script setup lang="ts">
defineProps<{ label?: string }>()
</script>

<template><span /></template>
"#;

fn write_tsconfig(project_root: &Path, exact_optional: bool) {
    let exact = if exact_optional {
        "\n    \"exactOptionalPropertyTypes\": true,"
    } else {
        ""
    };
    std::fs::write(
        project_root.join("tsconfig.json"),
        cstr!(
            r#"{{
  "compilerOptions": {{
    "strict": true,{exact}
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "types": [],
    "noEmit": true
  }},
  "include": ["src/**/*"]
}}"#
        )
        .as_str(),
    )
    .unwrap();
}

fn diagnostics_of(
    name: &str,
    parent: &str,
    exact_optional: bool,
) -> Option<Vec<(String, Option<u32>, String)>> {
    let project_root = create_project_case(
        name,
        &[("src/Child.vue", CHILD), ("src/Parent.vue", parent)],
    );
    write_tsconfig(&project_root, exact_optional);
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    snapshot
}

/// vue-tsc 3.3.4:
///
/// ```text
/// src/Parent.vue(6,12): error TS2379: Argument of type '{ label: undefined; }' is not assignable to parameter of type '{ readonly label?: string; } & VNodeProps & AllowedComponentProps & ComponentCustomProps & Record<string, unknown>' with 'exactOptionalPropertyTypes: true'.
/// ```
///
/// vize reports the same defect at the same place with a different **code**:
/// `TS2345`, not `TS2379`. That is a compiler-version difference, not a mapping
/// one — `tsc 6.0.3` (which `vue-tsc` pins) answers `TS2379` and the
/// `@typescript/native-preview` build vize runs answers `TS2345` for identical
/// code against an identical target, across every target shape tried including
/// `vue-tsc`'s own. The assertion is therefore on the position and the defect,
/// not on the code the runtime chose.
#[test]
fn an_explicitly_undefined_optional_prop_is_reported() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let Some(snapshot) = diagnostics_of(
        "exact-optional-props",
        r#"<script setup lang="ts">
import Child from './Child.vue'
const maybe: string | undefined = undefined
</script>

<template>
  <Child :label="maybe" />
</template>
"#,
        true,
    ) else {
        return;
    };

    assert_eq!(
        snapshot.len(),
        1,
        "expected exactly one diagnostic: {snapshot:?}"
    );
    let (file, _, message) = &snapshot[0];
    assert_eq!(file.as_str(), "src/Parent.vue");
    // Column 4 is the `C` of `<Child`, one byte right of the `<`. vue-tsc
    // anchors a whole-props failure on the element name: its oracle reads
    // `src/Parent.vue(6,12)` for `<template><Child …`, where 12 is the `C`.
    assert!(
        message.starts_with("7:4:error"),
        "the diagnostic anchors on the element name, got: {message}"
    );
    assert!(
        message.contains("exactOptionalPropertyTypes"),
        "the diagnostic must name the option that caused it, got: {message}"
    );
}

/// Without the option the check is inert, because an optional property accepts
/// `undefined` implicitly. `vue-tsc` reports nothing here either.
#[test]
fn the_same_binding_is_clean_without_the_option() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let Some(snapshot) = diagnostics_of(
        "exact-optional-props-off",
        r#"<script setup lang="ts">
import Child from './Child.vue'
const maybe: string | undefined = undefined
</script>

<template>
  <Child :label="maybe" />
</template>
"#,
        false,
    ) else {
        return;
    };

    assert_eq!(
        snapshot,
        vec![],
        "the option is off, so nothing is reported"
    );
}

/// The shapes that must stay silent under the option. Each one would be a false
/// positive on correct code:
///
/// * a prop the template did not pass, which `Child` declares optional;
/// * `class`, `style` and `data-*` fallthrough attributes the child never
///   declares;
/// * a prop whose declared type includes `null`, passed `null`;
/// * a prop whose declared type includes `undefined` explicitly — the child
///   opted into it.
#[test]
fn correct_usages_stay_silent_under_the_option() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "exact-optional-props-no-false-positive",
        &[
            (
                "src/Child.vue",
                r#"<script setup lang="ts">
defineProps<{
  label?: string
  nullish?: string | null
  explicit?: string | undefined
}>()
</script>

<template><span /></template>
"#,
            ),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Child from './Child.vue'
const maybe: string | undefined = undefined
const orNull: string | null = null
</script>

<template>
  <Child />
  <Child :label="'ok'" class="a" style="color: red" data-id="1" />
  <Child :nullish="orNull" />
  <Child :explicit="maybe" />
</template>
"#,
            ),
        ],
    );
    write_tsconfig(&project_root, true);
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(snapshot, vec![], "no correct usage may be reported");
}

/// An ordinary type mismatch is reported exactly once. Both checks see it — the
/// per-prop assignment and TypeScript's elaboration of the props literal — but
/// both land on the authored attribute name with the same code and the same
/// message, so `dedup_diagnostics` collapses them into one row.
#[test]
fn a_wrongly_typed_prop_is_reported_exactly_once() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let Some(snapshot) = diagnostics_of(
        "exact-optional-props-single-report",
        r#"<script setup lang="ts">
import Child from './Child.vue'
</script>

<template>
  <Child :label="42" />
</template>
"#,
        true,
    ) else {
        return;
    };

    assert_eq!(
        snapshot.len(),
        1,
        "a wrongly typed prop must be reported once, not once per check: {snapshot:?}"
    );
    assert_eq!(snapshot[0].1, Some(2322));
}
