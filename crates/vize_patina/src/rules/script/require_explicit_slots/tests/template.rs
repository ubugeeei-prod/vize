//! Template half (#3414 B): a `<slot>` the `defineSlots` type does not cover.
//!
//! These exercise the SFC engine path that supplies the parsed `<template>`
//! AST to the rule via [`super::lint_sfc`]. Because this half *creates*
//! findings from template evidence, the over-match probes below are as
//! load-bearing as the positive cases: each asserts the full finding set, and
//! the negative ones assert it is exactly empty.

use super::{findings, lint_sfc, none, owned, undeclared};

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
fn reports_the_whole_non_self_closing_slot_element_like_upstream() {
    let sfc = r#"<script setup lang="ts">
const ready = true;
</script>

<template>
  <slot></slot>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, "<slot></slot>", "default")]
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
fn skips_only_the_dynamic_slot_name() {
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown }>();
const which = 'footer';
</script>

<template>
  <div><slot :name="which" /><slot name="footer" /></div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, r#"<slot name="footer" />"#, "footer")]
    );
}

#[test]
fn treats_an_argumentless_v_bind_slot_as_the_declared_default_outlet() {
    let sfc = r#"<script setup lang="ts">
defineSlots<{ default(): unknown }>();
const attrs = { name: 'footer' };
</script>

<template>
  <div><slot v-bind="attrs" /></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn a_local_type_reports_only_an_undeclared_slot() {
    let sfc = r#"<script setup lang="ts">
type Slots = { default(): unknown; footer(): unknown };
defineSlots<Slots>();
</script>

<template>
  <div><slot name="other" /></div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, r#"<slot name="other" />"#, "other")]
    );
}

#[test]
fn a_local_interface_contributes_its_declared_slot_names() {
    let sfc = r#"<script setup lang="ts">
export interface Slots {
  default?: () => unknown;
  title?: () => unknown;
}
defineSlots<Slots>();
</script>

<template>
  <div><slot /><slot name="title" /></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn an_interface_in_the_normal_script_contributes_slots_to_script_setup() {
    let sfc = r#"<script lang="ts">
export interface Slots<T = unknown> {
  default?(props: { value: T }): unknown;
  title?(): unknown;
}
</script>
<script setup lang="ts" generic="T">
defineSlots<Slots<T>>();
</script>

<template>
  <div><slot /><slot name="title" /></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn an_index_signature_contributes_no_known_slot_names() {
    let sfc = r#"<script setup lang="ts">
defineSlots<{ [name: string]: () => unknown }>();
</script>

<template>
  <div><slot name="footer" /></div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, r#"<slot name="footer" />"#, "footer")]
    );
}

#[test]
fn an_intersection_keeps_inline_slot_names() {
    let sfc = r#"<script setup lang="ts">
defineSlots<ImportedSlots & { default(): unknown }>();
</script>

<template>
  <slot />
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn accepts_a_string_literal_bound_slot_name() {
    let sfc = r#"<script setup lang="ts">
defineSlots<{ footer(): unknown }>();
</script>

<template>
  <slot :name="'footer'" />
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn reports_a_template_slot_when_no_declaration_exists_at_all() {
    let sfc = r#"<script setup lang="ts">
const id: number = 1;
</script>

<template>
  <div><slot name="footer" /></div>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, r#"<slot name="footer" />"#, "footer")]
    );
}

// --- The pre-existing script-only subset must keep working -----------------

#[test]
fn reports_the_slot_outlet_instead_of_the_use_slots_call() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ id: number }>();
const slots = useSlots();
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
