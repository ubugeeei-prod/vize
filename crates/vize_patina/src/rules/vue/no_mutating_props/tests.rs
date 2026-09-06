use super::NoMutatingProps;
use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleCategory};

mod script_mutations;
mod template_handlers;

/// Lint a full SFC with only this rule enabled.
///
/// `vue/no-mutating-props` is a member of `SEMANTIC_TEMPLATE_RULES`, so
/// enabling it alone still produces the croquis analysis the rule reads.
fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec!["vue/no-mutating-props".into()]))
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

fn span_for(sfc: &str, needle: &str) -> (u32, u32) {
    let start = sfc.find(needle).expect("expected source span");
    (start as u32, (start + needle.len()) as u32)
}

fn last_span_for(sfc: &str, needle: &str) -> (u32, u32) {
    let start = sfc.rfind(needle).expect("expected source span");
    (start as u32, (start + needle.len()) as u32)
}

#[test]
fn test_meta() {
    let rule = NoMutatingProps;
    assert_eq!(rule.meta().name, "vue/no-mutating-props");
    assert_eq!(rule.meta().category, RuleCategory::Essential);
    assert_eq!(rule.meta().default_severity, Severity::Error);
}

// --- Over-match probes: none of these may manufacture a finding ------------

#[test]
fn ignores_a_prop_assignment_inside_an_html_comment() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <!-- msg = 'x' -->
  <button @click="void 0">go</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_prop_assignment_in_a_text_node_or_plain_attribute() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <p title="msg = 'x'">msg = 'x'</p>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_prop_assignment_inside_a_v_pre_region() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <pre v-pre>{{ msg = 1 }}</pre>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

// --- The pre-existing v-model half must keep working ----------------------

#[test]
fn reports_v_model_bound_to_a_prop() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <input v-model="msg" />
</template>
"#;
    let (start, end) = last_span_for(sfc, "msg");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            start,
            end,
            "Unexpected mutation of prop 'msg' via v-model",
        )]
    );
}
