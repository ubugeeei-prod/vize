use super::{NoMutatingProps, NoMutatingPropsOptions};
use crate::diagnostic::Severity;
use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleCategory, RuleRegistry};

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

fn lint_sfc_with_rule(sfc: &str, rule: NoMutatingProps) -> LintResult {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(rule));
    Linter::with_registry(registry).lint_sfc(sfc, "Probe.vue")
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
    let rule = NoMutatingProps::default();
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

#[test]
fn reports_deep_v_model_on_runtime_and_untyped_props() {
    for sfc in [
        r#"<script setup>const props = defineProps({ store: Object, storeProp: String })</script>
<template><draggable v-model="store[storeProp].items" /></template>"#,
        r#"<script setup lang="ts">defineProps<{ modelValue }>()</script>
<template><Checkbox v-model="modelValue.repeat" /></template>"#,
    ] {
        let result = lint_sfc(sfc);
        assert_eq!(
            result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_name == "vue/no-mutating-props")
                .count(),
            1,
            "{sfc}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn reports_deep_v_model_on_multiline_runtime_props() {
    let sfc = r#"<template>
  <draggable v-model="store[storeProp].items" />
</template>
<script setup>
const props = defineProps({
  store: {
    type: Object,
    default: null,
  },
  storeProp: {
    type: String,
    default: '',
  },
})
</script>"#;
    let result = lint_sfc(sfc);
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/no-mutating-props")
            .count(),
        1,
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn reports_runtime_prop_v_model_across_table_foster_parenting() {
    let sfc = r#"<template>
  <table>
    <draggable v-model="store[storeProp].items" />
  </table>
</template>
<script setup>
const props = defineProps({
  store: Object,
  storeProp: String,
})
</script>"#;
    let (start, end) = span_for(sfc, "store[storeProp].items");
    assert_eq!(
        findings(&lint_sfc(sfc)),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            start,
            end,
            "Unexpected mutation of prop 'store[storeProp].items' via v-model",
        )]
    );
}

#[test]
fn local_v_model_refs_shadow_same_named_runtime_props() {
    let sfc = r#"<template><ShareModal v-model:embed-options="embedOptions" /></template>
<script setup>
const props = defineProps({ embedOptions: Object })
const embedOptions = useVModel(props, "embedOptions")
</script>"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn local_reactive_and_ref_bindings_shadow_runtime_props() {
    let sfc = r#"<template>
  <FormControl v-model="condition[1]" />
  <FormControl v-model="filter.value" />
  <input v-model="subject" />
</template>
<script setup>
const props = defineProps({ condition: Array, filter: Object, subject: String })
const condition = reactive(props.condition)
const filter = reactive(props.filter)
const subject = ref(props.subject)
</script>"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn template_region_disable_suppresses_authored_fallback_reports() {
    let sfc = r#"<template>
<!-- eslint-disable vue/no-mutating-props -->
<MkSwitch v-model="props.modelValue.detailed" />
</template>
<script setup>
const props = defineProps<{ modelValue: { detailed: boolean } }>()
</script>"#;
    assert_eq!(findings(&lint_sfc(sfc)), none());
}

#[test]
fn shallow_only_reports_direct_template_mutations_but_allows_nested_values() {
    let sfc = r#"<script setup lang="ts">
const props = defineProps<{ user: { name: string }; count: number }>();
</script>

<template>
  <input v-model="props.user.name" />
  <button @click="props.user.name = 'Ada'; props.count = 1">go</button>
</template>
"#;
    let result = lint_sfc_with_rule(
        sfc,
        NoMutatingProps::new(NoMutatingPropsOptions { shallow_only: true }),
    );
    let (start, end) = span_for(sfc, "props.count = 1");
    assert_eq!(
        findings(&result),
        vec![(
            "vue/no-mutating-props",
            Severity::Error,
            start,
            end,
            "Unexpected mutation of prop 'props.count' in an inline handler",
        )]
    );
}
