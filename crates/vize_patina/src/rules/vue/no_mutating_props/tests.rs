use super::NoMutatingProps;
use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleCategory};

mod script_mutations;

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

#[test]
fn test_meta() {
    let rule = NoMutatingProps;
    assert_eq!(rule.meta().name, "vue/no-mutating-props");
    assert_eq!(rule.meta().category, RuleCategory::Essential);
    assert_eq!(rule.meta().default_severity, Severity::Error);
}

// --- The recovered case: an assignment inside an inline handler ------------

#[test]
fn reports_prop_assignment_in_an_inline_handler() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ msg: string }>();
</script>

<template>
  <button @click="props.msg = 'x'">{{ props.msg }}</button>
</template>
"#;
    let directive = sfc.find("@click=").expect("handler directive");
    let directive_end = sfc.find(">{{").expect("end of the start tag");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            directive as u32,
            directive_end as u32,
            "Unexpected mutation of prop 'props.msg' in an inline handler",
        )]
    );
}

#[test]
fn reports_bare_prop_assignment_in_an_inline_handler() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <button @click="msg = 'x'">go</button>
</template>
"#;
    let directive = sfc.find("@click=").expect("handler directive");
    let directive_end = sfc.find(">go").expect("end of the start tag");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            directive as u32,
            directive_end as u32,
            "Unexpected mutation of prop 'msg' in an inline handler",
        )]
    );
}

#[test]
fn reports_a_prop_update_expression() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ count: number }>();
</script>

<template>
  <button @click="props.count++">go</button>
</template>
"#;
    let directive = sfc.find("@click=").expect("handler directive");
    let directive_end = sfc.find(">go").expect("end of the start tag");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            directive as u32,
            directive_end as u32,
            "Unexpected mutation of prop 'props.count' in an inline handler",
        )]
    );
}

#[test]
fn reports_each_distinct_prop_of_a_multi_statement_handler_once() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string; count: number }>();
</script>

<template>
  <button @click="msg = 'x'; count = 1; msg = 'y'">go</button>
</template>
"#;
    let directive = sfc.find("@click=").expect("handler directive");
    let directive_end = sfc.find(">go").expect("end of the start tag");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![
            (
                "vue/no-mutating-props",
                Severity::Error,
                directive as u32,
                directive_end as u32,
                "Unexpected mutation of prop 'msg' in an inline handler",
            ),
            (
                "vue/no-mutating-props",
                Severity::Error,
                directive as u32,
                directive_end as u32,
                "Unexpected mutation of prop 'count' in an inline handler",
            ),
        ]
    );
}

// --- The template does not mutate a prop: exactly zero findings ------------

#[test]
fn ignores_a_handler_that_reads_a_prop() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ msg: string }>();
const emit = defineEmits<{ go: [string] }>();
</script>

<template>
  <button @click="emit('go', props.msg)">{{ props.msg }}</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_assignment_to_local_state() {
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue';
defineProps<{ msg: string }>();
const draft = ref('');
</script>

<template>
  <button @click="draft = 'x'">go</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
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
fn ignores_a_prop_assignment_inside_a_string_literal() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <button @click="console.log('msg = 1')">go</button>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_an_identifier_that_merely_starts_with_a_prop_name() {
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue';
defineProps<{ msg: string }>();
const msgExtra = ref('');
</script>

<template>
  <button @click="msgExtra = 'x'">go</button>
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

#[test]
fn ignores_a_v_for_alias_that_shadows_a_prop_name() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string; rows: string[] }>();
</script>

<template>
  <ul>
    <li v-for="msg in rows" :key="msg">
      <button @click="msg = 'x'">go</button>
    </li>
  </ul>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn ignores_a_slot_variable_that_shadows_a_prop_name() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string }>();
</script>

<template>
  <Child v-slot="{ msg }">
    <button @click="msg = 'x'">go</button>
  </Child>
</template>
"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn still_reports_a_prop_after_a_shadowing_subtree_ends() {
    let sfc = r#"<script setup lang="ts">
defineProps<{ msg: string; rows: string[] }>();
</script>

<template>
  <ul>
    <li v-for="msg in rows" :key="msg">
      <button @click="msg = 'x'">shadowed</button>
    </li>
  </ul>
  <button @click="msg = 'y'">not shadowed</button>
</template>
"#;
    let directive = sfc.rfind("@click=").expect("handler directive");
    let directive_end = sfc.rfind(">not").expect("end of the start tag");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            directive as u32,
            directive_end as u32,
            "Unexpected mutation of prop 'msg' in an inline handler",
        )]
    );
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
    let directive = sfc.find("v-model=").expect("model directive");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            directive as u32,
            (directive + "v-model=\"msg\"".len()) as u32,
            "Unexpected mutation of prop 'msg' via v-model",
        )]
    );
}
