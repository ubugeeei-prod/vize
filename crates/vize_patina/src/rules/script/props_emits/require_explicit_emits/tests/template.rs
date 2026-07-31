//! Template half (#3413 A): a template `$emit` of an undeclared event.
//!
//! Because this half *creates* findings from template evidence, the over-match
//! probes are as load-bearing as the positive cases: each asserts the full
//! finding set, and the negative ones assert it is exactly empty.

use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};

/// Lint a full SFC end-to-end with only this rule enabled, exercising the
/// engine path that supplies the parsed `<template>` AST to the rule.
fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec!["script/require-explicit-emits".into()]))
        .lint_sfc(sfc, "Probe.vue")
}

/// The full identity of every finding: rule, severity, byte range, message.
fn findings(result: &LintResult) -> Vec<(&'static str, Severity, u32, u32, &str)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.severity,
                diagnostic.start,
                diagnostic.end,
                diagnostic.message.as_str(),
            )
        })
        .collect()
}

fn none() -> Vec<(&'static str, Severity, u32, u32, &'static str)> {
    Vec::new()
}

/// The finding the quoted event name `literal` produces, located inside the
/// `<template>` block.
fn undeclared(
    sfc: &str,
    literal: &str,
    name: &str,
) -> (&'static str, Severity, u32, u32, std::string::String) {
    let template = sfc.find("<template>").expect("template block");
    let start = template + sfc[template..].find(literal).expect("event literal");
    (
        "script/require-explicit-emits",
        Severity::Warning,
        start as u32,
        (start + literal.len()) as u32,
        format!("The emitted event '{name}' is not declared in defineEmits or the emits option."),
    )
}

/// [`findings`] with owned messages, so it compares against [`undeclared`].
fn owned(result: &LintResult) -> Vec<(&'static str, Severity, u32, u32, std::string::String)> {
    findings(result)
        .into_iter()
        .map(|(rule, severity, start, end, message)| {
            (rule, severity, start, end, message.to_string())
        })
        .collect()
}

// --- The recovered case: a template `$emit` of an undeclared event ---------

#[test]
fn reports_an_undeclared_dollar_emit_issue_3413() {
    // Exact reproduction from #3413 A.
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>

<template>
  <button @click="$emit('cancel')">x</button>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, "'cancel'", "cancel")]
    );
}

#[test]
fn reports_an_undeclared_call_of_the_captured_binding() {
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
</script>

<template>
  <button @click="emit('cancel')">x</button>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, "'cancel'", "cancel")]
    );
}

#[test]
fn reports_an_undeclared_dollar_emit_against_the_options_api_emits() {
    let sfc = r#"<script>
export default { emits: ['save'] };
</script>

<template>
  <button @click="$emit('cancel')">x</button>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, "'cancel'", "cancel")]
    );
}

#[test]
fn reports_an_undeclared_dollar_emit_in_an_interpolation() {
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>

<template>
  <span>{{ $emit('cancel') }}</span>
</template>
"#;
    assert_eq!(
        owned(&lint_sfc(sfc)),
        vec![undeclared(sfc, "'cancel'", "cancel")]
    );
}

// --- The event is declared: exactly zero findings --------------------------

#[test]
fn ignores_a_declared_dollar_emit() {
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>

<template>
  <button @click="$emit('save')">x</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_dynamic_event_name() {
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
const name = 'cancel'
</script>

<template>
  <button @click="$emit(name)">x</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

// --- The soundness guard: no declaration means no report ------------------

#[test]
fn ignores_a_dollar_emit_when_nothing_is_declared() {
    // The rule reports only when a declaration exists and is fully known;
    // otherwise the events may be declared elsewhere, or intentionally not at
    // all, and flagging would be a false positive.
    let sfc = r#"<script setup lang="ts">
const a = 1;
</script>

<template>
  <button @click="$emit('cancel')">{{ a }}</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_dollar_emit_when_the_declaration_is_not_enumerable() {
    let sfc = r#"<script setup lang="ts">
const names = ['save'];
const emit = defineEmits([...names]);
</script>

<template>
  <button @click="$emit('cancel')">x</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

// --- Over-match probes: none of these may manufacture a finding ------------

#[test]
fn ignores_a_dollar_emit_inside_an_html_comment() {
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>

<template>
  <div><!-- $emit('cancel') --></div>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_dollar_emit_in_a_text_node_or_plain_attribute() {
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>

<template>
  <p title="$emit('cancel')">$emit('cancel')</p>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_dollar_emit_inside_a_string_literal() {
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>

<template>
  <button @click="console.log('$emit(\'cancel\')')">x</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_dollar_emit_inside_a_v_pre_region() {
    let sfc = r#"<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>

<template>
  <pre v-pre>{{ $emit('cancel') }}</pre>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_identifier_that_merely_ends_with_the_binding_name() {
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
const myemit = (name) => name
</script>

<template>
  <button @click="myemit('cancel')">x</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_member_dollar_emit_on_another_instance() {
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
</script>

<template>
  <Child ref="child" @click="child.$emit('cancel')" />
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_slot_variable_that_shadows_the_captured_binding() {
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
</script>

<template>
  <Child v-slot="{ emit }"><button @click="emit('cancel')">x</button></Child>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_template_dollar_emit_when_a_sibling_script_block_exists() {
    // Each block is linted separately, so neither sees the whole declared set.
    let sfc = r#"<script>
export default { emits: ['cancel'] };
</script>

<script setup>
const emit = defineEmits(['save'])
</script>

<template>
  <button @click="$emit('cancel')">x</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

// --- The pre-existing script-only subset must keep working -----------------

#[test]
fn still_reports_a_script_side_emit_through_the_sfc_path() {
    let sfc = r#"<script setup>
const emit = defineEmits(['save'])
function onClick() {
  emit('cancel')
}
</script>

<template>
  <button @click="onClick">x</button>
</template>
"#;
    let literal = sfc.find("'cancel'").expect("emit literal");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "script/require-explicit-emits",
            Severity::Warning,
            literal as u32,
            (literal + "'cancel'".len()) as u32,
            "The emitted event 'cancel' is not declared in defineEmits or the emits option.",
        )]
    );
}
