//! An ecosystem-sized props type must stay inside TypeScript's complexity
//! limits (#3569).
//!
//! Oracle: TypeScript 6.0.3 and Vue 3.6.0-beta.10.
//!
//! The first attempt at #3569 inferred the authored object as a generic `A` and
//! chose the whole-props target with conditional and mapped types computed over
//! the child's entire props type. On a real component that reported `TS2590:
//! Expression produces a union type that is too complex to represent`, which is
//! worse than the bug it fixed: the compiler gives up and every genuine
//! diagnostic in the file disappears with it. The check now targets the child's
//! raw props marker directly, so nothing it emits grows with the props type —
//! this pins that, and pins that the #3569 diagnostic still lands on a child
//! this size.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::{String, append, cstr};

/// A child whose props type is the size of a real design-system component:
/// literal unions, callbacks, arrays and camelCase names that also have a kebab
/// spelling, plus a typed `emits` block. 121 declared props in total.
fn wide_child_source() -> String {
    let mut source = String::from("<script setup lang=\"ts\">\ndefineProps<{\n  title: string\n");
    for index in 0..40u32 {
        append!(
            source,
            "  variant{index}?: 'solid' | 'outline' | 'ghost' | 'link'\n"
        );
        append!(
            source,
            "  onSelect{index}?: (value: string, index: number) => void\n"
        );
        append!(
            source,
            "  itemList{index}?: readonly {{ id: string; label: string }}[]\n"
        );
    }
    source.push_str(
        "}>()\ndefineEmits<{ change: [value: string]; close: [] }>()\n</script>\n<template><span /></template>\n",
    );
    source
}

#[test]
fn a_wide_props_type_reports_the_missing_prop_without_a_complexity_error() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "wide-props-type-complexity",
        &[
            ("src/Wide.vue", wide_child_source().as_str()),
            (
                "src/Parent.vue",
                r#"<script setup lang="ts">
import Wide from './Wide.vue'
const pick = (_value: string, _index: number) => {}
</script>

<template>
  <Wide variant0="solid" :on-select3="pick" class="wide" />
  <Wide title="ok" variant0="solid" :on-select3="pick" class="wide" />
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

    // One row, `TS2345`, not `TS2590`: the first usage omits `title` and the
    // second one passes it. A compiler that gave up would show here as a missing
    // row plus a complexity error. The readonly flatten (#3890) keeps the wide
    // instantiation inside the checker's limits, and the display elides the
    // middle members instead of overflowing.
    let authored = "{ variant0: \"solid\"; onSelect3: (_value: string, _index: number) => void; class: string; }";
    let flattened = "{ readonly title: string; readonly variant0?: \"ghost\" | \"link\" | \"outline\" | \"solid\" | undefined; readonly onSelect0?: ((value: string, index: number) => void) | undefined; readonly itemList0?: readonly { ...; }[] | undefined; ... 118 more ...; readonly onClose?: (() => any) | undefined; }";
    // The isolated stub's `NativeElements` is `Record<string, …>`, so the
    // #4966 global-attr surface renders as its open index-map form here.
    let native_tail = "__VizePublicComponentAttrs & { [x: string]: unknown; } & { [x: `data${string}`]: unknown; } & Record<string, unknown>";
    assert_eq!(
        snapshot,
        vec![(
            String::from("src/Parent.vue"),
            Some(2345),
            cstr!(
                "7:4:error Argument of type '{authored}' is not assignable to parameter of type '__VizeComponentCheckProps<__Wide_CheckProps_0, {native_tail}>'.\nProperty 'title' is missing in type '{authored}' but required in type '{flattened}'."
            ),
        )],
        "a 121-prop child reports exactly its missing required prop, with no complexity error"
    );
}
