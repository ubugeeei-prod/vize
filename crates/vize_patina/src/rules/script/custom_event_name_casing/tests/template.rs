//! Template half (#3413 B): a template `$emit` with a non-camelCase name.
//!
//! Because this half *creates* findings from template evidence, the over-match
//! probes are as load-bearing as the positive cases: each asserts the full
//! finding set, and the negative ones assert it is exactly empty.

use super::{findings, lint_sfc, none};
use crate::diagnostic::Severity;

/// The finding the quoted event name `literal` produces, located inside the
/// `<template>` block.
fn not_camel_case(
    sfc: &str,
    literal: &str,
    name: &str,
) -> (&'static str, Severity, u32, u32, std::string::String) {
    let template = sfc.find("<template>").expect("template block");
    let start = template + sfc[template..].find(literal).expect("event literal");
    (
        "script/custom-event-name-casing",
        Severity::Error,
        start as u32,
        (start + literal.len()) as u32,
        format!("Custom event name '{name}' is not camelCase."),
    )
}

/// [`findings`] with owned messages, so it compares against [`not_camel_case`].
fn owned(
    result: &crate::linter::LintResult,
) -> Vec<(&'static str, Severity, u32, u32, std::string::String)> {
    findings(result)
        .into_iter()
        .map(|(rule, severity, start, end, message)| {
            (rule, severity, start, end, message.to_string())
        })
        .collect()
}

// --- The recovered case: a template `$emit` with a bad name ----------------

#[test]
fn reports_a_kebab_case_dollar_emit_issue_3413() {
    // Exact reproduction from #3413 B.
    let sfc = r#"<script setup lang="ts">
const a = 1;
</script>

<template>
  <button @click="$emit('foo-bar')">{{ a }}</button>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![not_camel_case(sfc, "'foo-bar'", "foo-bar")]
    );
}

#[test]
fn reports_a_pascal_case_dollar_emit() {
    let sfc = r#"<script setup lang="ts">
const a = 1;
</script>

<template>
  <button @click="$emit('FooBar')">{{ a }}</button>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![not_camel_case(sfc, "'FooBar'", "FooBar")]
    );
}

#[test]
fn reports_a_kebab_case_call_of_the_captured_binding() {
    // The captured binding is template-visible and dispatches the same events.
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ 'foo-bar': [] }>();
</script>

<template>
  <button @click="emit('foo-bar')">go</button>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![not_camel_case(sfc, "'foo-bar'", "foo-bar")]
    );
}

#[test]
fn reports_a_dollar_emit_in_an_interpolation() {
    let sfc = r#"<script setup lang="ts">
const a = 1;
</script>

<template>
  <span>{{ $emit('foo-bar') || a }}</span>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![not_camel_case(sfc, "'foo-bar'", "foo-bar")]
    );
}

// --- Well-named events: exactly zero findings ------------------------------

#[test]
fn ignores_a_camel_case_dollar_emit() {
    let sfc = r#"<script setup lang="ts">
const a = 1;
</script>

<template>
  <button @click="$emit('fooBar')">{{ a }}</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_the_v_model_update_prefix() {
    let sfc = r#"<script setup lang="ts">
const a = 1;
</script>

<template>
  <input @input="$emit('update:modelValue', a)" />
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_dynamic_event_name() {
    let sfc = r#"<script setup lang="ts">
const name = 'foo-bar';
</script>

<template>
  <button @click="$emit(name)">go</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

// --- Over-match probes: none of these may manufacture a finding ------------

#[test]
fn ignores_a_dollar_emit_inside_an_html_comment() {
    let sfc = r#"<script setup lang="ts">
const a = 1;
</script>

<template>
  <div>{{ a }}<!-- $emit('foo-bar') --></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_dollar_emit_in_a_text_node_or_plain_attribute() {
    let sfc = r#"<script setup lang="ts">
const a = 1;
</script>

<template>
  <p title="$emit('foo-bar')">$emit('foo-bar') {{ a }}</p>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_dollar_emit_inside_a_string_literal() {
    let sfc = r#"<script setup lang="ts">
const a = 1;
</script>

<template>
  <button @click="console.log('$emit(\'foo-bar\')')">{{ a }}</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_dollar_emit_inside_a_v_pre_region() {
    let sfc = r#"<script setup lang="ts">
const a = 1;
</script>

<template>
  <pre v-pre>{{ $emit('foo-bar') }}</pre>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_identifier_that_merely_ends_with_emit() {
    let sfc = r#"<script setup lang="ts">
const myEmit = (name: string) => name;
</script>

<template>
  <button @click="myEmit('foo-bar')">go</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_member_dollar_emit_on_another_instance() {
    let sfc = r#"<script setup lang="ts">
const a = 1;
</script>

<template>
  <Child ref="child" @click="child.$emit('foo-bar')">{{ a }}</Child>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_slot_variable_that_shadows_the_captured_binding() {
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ 'foo-bar': [] }>();
</script>

<template>
  <Child v-slot="{ emit }"><button @click="emit('foo-bar')">go</button></Child>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_template_dollar_emit_when_a_sibling_script_block_exists() {
    // Each block is linted separately and a template `$emit` is visible to
    // both, so reporting it from either would double the diagnostic.
    let sfc = r#"<script>
export default { name: 'Probe' };
</script>

<script setup lang="ts">
const a = 1;
</script>

<template>
  <button @click="$emit('foo-bar')">{{ a }}</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

// --- The pre-existing script-only subset must keep working -----------------

#[test]
fn still_reports_a_script_side_emit_through_the_sfc_path() {
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ 'foo-bar': [] }>();
emit('foo-bar');
</script>

<template>
  <div>ok</div>
</template>
"#;
    let literal = sfc.rfind("'foo-bar'").expect("emit literal");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "script/custom-event-name-casing",
            Severity::Error,
            literal as u32,
            (literal + "'foo-bar'".len()) as u32,
            "Custom event name 'foo-bar' is not camelCase.",
        )]
    );
}
