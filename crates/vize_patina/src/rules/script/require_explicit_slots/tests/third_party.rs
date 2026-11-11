//! Imported wrappers, repeated names, and a bare `defineSlots()`.

use super::{lint_sfc, owned, undeclared};
use crate::linter::Linter;

#[test]
fn an_imported_method_interface_reports_the_undeclared_outlet() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("types.ts"),
        "export interface CardSlots {\n  default(): unknown;\n  footer(props: { count: number }): unknown;\n}\n",
    )
    .unwrap();
    let source = r#"<script setup lang="ts">
import type { CardSlots } from "./types";

defineSlots<CardSlots>();
</script>

<template>
  <div>
    <slot />
    <slot name="footer" :count="1" />
    <slot name="extra" />
  </div>
</template>
"#;
    let filename = dir.path().join("Imported.vue");
    let result = Linter::new()
        .with_enabled_rules(Some(vec!["script/require-explicit-slots".into()]))
        .lint_sfc(source, filename.to_str().unwrap());
    let messages: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(
        messages,
        ["Slot 'extra' is rendered in the template but not declared in defineSlots<...>()."]
    );
}

#[test]
fn a_bare_define_slots_declares_no_slots() {
    let sfc = r#"<script setup lang="ts">
defineSlots();
</script>

<template>
  <div>
    <slot />
    <slot name="footer" />
  </div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![
            undeclared(sfc, "<slot />", "default"),
            undeclared(sfc, r#"<slot name="footer" />"#, "footer"),
        ]
    );
}

#[test]
fn a_repeated_slot_name_in_one_type_is_reported() {
    for sfc in [
        r#"<script setup lang="ts">
defineSlots<{
  default(props: { msg: string }): unknown;
  default(props: { msg: string }): unknown;
}>();
</script>

<template>
  <div><slot msg="x" /></div>
</template>
"#,
        r#"<script setup lang="ts">
interface Slots {
  default(props: { msg: string }): unknown;
  default(props: { msg: string }): unknown;
}
defineSlots<Slots>();
</script>

<template>
  <div><slot msg="x" /></div>
</template>
"#,
    ] {
        let start = sfc
            .match_indices("default")
            .nth(1)
            .expect("second default")
            .0;
        assert_eq!(
            owned(&lint_sfc(sfc)),
            vec![(
                "script/require-explicit-slots",
                crate::diagnostic::Severity::Warning,
                start as u32,
                (start + "default".len()) as u32,
                "Slot 'default' is already defined.".to_string(),
            )]
        );
    }
}

#[test]
fn a_local_utility_wrapper_keeps_the_inner_slot_names() {
    let cases = [
        r#"<script setup lang="ts">
interface CardSlots {
  default(): unknown;
  footer(props: { count: number }): unknown;
}
defineSlots<Readonly<CardSlots>>();
</script>

<template>
  <div><slot /><slot name="footer" /><slot name="extra" /></div>
</template>
"#,
        r#"<script setup lang="ts">
interface CardSlots {
  default(): unknown;
  footer(props: { count: number }): unknown;
}
defineSlots<Partial<CardSlots>>();
</script>

<template>
  <div><slot /><slot name="footer" /><slot name="extra" /></div>
</template>
"#,
        r#"<script setup lang="ts">
defineSlots<Record<"default" | "footer", () => unknown>>();
</script>

<template>
  <div><slot /><slot name="footer" /><slot name="extra" /></div>
</template>
"#,
    ];
    for sfc in cases {
        assert_eq!(
            owned(&lint_sfc(sfc)),
            vec![undeclared(sfc, r#"<slot name="extra" />"#, "extra")]
        );
    }
}
