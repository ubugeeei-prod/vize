//! Template half (#3414 B): a `<slot>` the `defineSlots` type does not cover.
//!
//! These exercise the SFC engine path that supplies the parsed `<template>`
//! AST to the rule via [`super::lint_sfc`]. Because this half *creates*
//! findings from template evidence, the over-match probes below are as
//! load-bearing as the positive cases: each asserts the full finding set, and
//! the negative ones assert it is exactly empty.

use super::{findings, lint_sfc, none, owned, undeclared};
use crate::diagnostic::Severity;

// --- The recovered case: a rendered slot the declaration omits --------------

#[test]
fn reports_a_slot_missing_from_the_declared_set_issue_3414() {
    // Exact reproduction from #3414 B.
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown }>();
</script>

<template>
  <div><slot /><slot name="footer" /></div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, r#"<slot name="footer" />"#, "footer")]
    );
}

#[test]
fn reports_an_undeclared_default_slot() {
    // A `<slot>` with no `name` renders the default slot.
    let sfc = r#"<script setup lang="ts">
defineSlots<{ footer(): unknown }>();
</script>

<template>
  <div><slot /></div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, "<slot />", "default")]
    );
}

#[test]
fn reports_a_slot_nested_in_another_components_slot_content() {
    // A `<slot>` inside a child component's default slot is still *this*
    // component's outlet, so it is checked against this component's contract.
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown }>();
</script>

<template>
  <Child v-slot="{ footer }"><slot name="footer" /></Child>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, r#"<slot name="footer" />"#, "footer")]
    );
}

#[test]
fn reports_every_undeclared_slot_in_source_order() {
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown }>();
</script>

<template>
  <div>
    <slot name="header" />
    <slot name="footer" />
  </div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![
            undeclared(sfc, r#"<slot name="header" />"#, "header"),
            undeclared(sfc, r#"<slot name="footer" />"#, "footer"),
        ]
    );
}

#[test]
fn reports_a_string_literal_slot_name_against_a_string_literal_declaration() {
    let sfc = r#"<script setup lang="ts">
defineSlots<{ 'my-slot'(): unknown }>();
</script>

<template>
  <div><slot name="my-slot" /><slot name="other" /></div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, r#"<slot name="other" />"#, "other")]
    );
}

// --- Every rendered slot is declared: exactly zero findings ----------------

#[test]
fn ignores_a_template_whose_slots_are_all_declared() {
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown; footer(): unknown }>();
</script>

<template>
  <div><slot /><slot name="footer" /></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_property_signature_declaration() {
    // `default: () => any` declares the same slot as `default(): unknown`.
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default: () => unknown }>();
</script>

<template>
  <div><slot /></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

// --- Over-match probes: none of these may manufacture a finding ------------

#[test]
fn ignores_a_slot_inside_an_html_comment() {
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown }>();
</script>

<template>
  <div><slot /><!-- <slot name="footer" /> --></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_slot_name_in_a_text_node_or_plain_attribute() {
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown }>();
</script>

<template>
  <p title="&lt;slot name=&quot;footer&quot; /&gt;">slot name="footer"</p>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_slot_inside_a_v_pre_region() {
    // `v-pre` skips compilation, so the `<slot>` renders as a literal tag
    // rather than as an outlet of this component.
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown }>();
</script>

<template>
  <pre v-pre><slot name="footer" /></pre>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_tag_whose_name_merely_starts_with_slot() {
    // `<slot-machine>` is a component, not a slot outlet.
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown }>();
</script>

<template>
  <div><slot /><slot-machine name="footer" /></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_the_whole_component_when_one_slot_name_is_dynamic() {
    // `:name` is resolved at runtime, so the rendered set is unknown and even
    // the statically undeclared `footer` must not be reported.
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown }>();
const which = 'footer';
</script>

<template>
  <div><slot :name="which" /><slot name="footer" /></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_the_whole_component_when_a_slot_spreads_a_bound_object() {
    // An argument-less `v-bind` can supply `name`.
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown }>();
const attrs = { name: 'footer' };
</script>

<template>
  <div><slot v-bind="attrs" /><slot name="footer" /></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_declaration_that_cannot_be_enumerated() {
    // `defineSlots<Slots>()` names a type this rule does not resolve, so the
    // declared set is unknown and a missing name proves nothing.
    let sfc = r#"<script setup lang="ts">
type Slots = { default(): unknown; footer(): unknown };
defineSlots<Slots>();
</script>

<template>
  <div><slot name="footer" /></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_declaration_with_an_index_signature() {
    let sfc = r#"<script setup lang="ts">
defineSlots<{ [name: string]: () => unknown }>();
</script>

<template>
  <div><slot name="footer" /></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_template_slot_when_no_declaration_exists_at_all() {
    // Without `defineSlots` every `<slot>` would be "undeclared"; that is the
    // `useSlots`-without-`defineSlots` case the script half owns, and it needs
    // a `useSlots()` call to fire.
    let sfc = r#"<script setup lang="ts">
const id: number = 1;
</script>

<template>
  <div><slot name="footer" /></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

// --- The pre-existing script-only subset must keep working -----------------

#[test]
fn still_reports_use_slots_without_define_slots_through_the_sfc_path() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ id: number }>();
const slots = useSlots();
</script>

<template>
  <div><slot /></div>
</template>
"#;
    let call = sfc.find("useSlots()").expect("useSlots call");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "script/require-explicit-slots",
            Severity::Warning,
            call as u32,
            (call + "useSlots()".len()) as u32,
            "Slots consumed via useSlots() must be explicitly typed with defineSlots<...>().",
        )]
    );
}

#[test]
fn still_reports_nothing_when_define_slots_covers_a_used_slots_call() {
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown }>();
const slots = useSlots();
</script>

<template>
  <div><slot /></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}
