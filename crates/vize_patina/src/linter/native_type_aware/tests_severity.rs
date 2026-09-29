use super::{
    LintPreset, Linter, RULE_NO_REACTIVITY_LOSS, RULE_REQUIRE_TYPED_PROPS, corsa_available,
    lint_sfc_with_corsa,
};

#[test]
fn type_aware_diagnostics_snapshot() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated).with_type_aware_lint(true);
    let source = r#"<script setup lang="ts">
import { ref } from 'vue'
defineProps(['msg'])
defineEmits(['save'])
const payload: any = { label: 'unsafe' }
const anyHandler: any = () => {}
const countRef = ref(0)
const count = countRef.value

async function loadData(): Promise<number> {
  return 1
}

loadData()
useMyComposable(count)
</script>

<template>
  <div>{{ payload.label }}</div>
  <button @click="anyHandler()">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "TypeAwareFixture.vue");
    let mut diagnostics = result
        .diagnostics
        .iter()
        .map(|diag| {
            (
                diag.rule_name,
                diag.message.as_str(),
                diag.start,
                diag.end,
                diag.help.as_deref(),
            )
        })
        .collect::<Vec<_>>();
    diagnostics.sort_unstable();
    insta::assert_debug_snapshot!(diagnostics);
}

#[test]
fn type_rule_severity_override_is_an_error() {
    let linter = Linter::new()
        .with_type_aware_lint(true)
        .with_rule_severity_overrides(vec![(
            RULE_REQUIRE_TYPED_PROPS.into(),
            crate::Severity::Error,
        )]);
    let source = "<script setup lang=\"ts\">\ndefineProps(['msg'])\n</script>\n";
    let result = linter.lint_sfc(source, "Fixture.vue");
    let typed: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE_REQUIRE_TYPED_PROPS)
        .collect();
    assert_eq!(typed.len(), 1);
    assert_eq!(typed[0].severity, crate::Severity::Error);
    assert_eq!(
        result.error_count,
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == crate::Severity::Error)
            .count()
    );
    assert_eq!(
        result.warning_count,
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == crate::Severity::Warning)
            .count()
    );
}

#[test]
fn type_rule_category_severity_override_is_an_error() {
    let linter = Linter::with_preset(LintPreset::Opinionated)
        .with_type_aware_lint(true)
        .with_category_severity_overrides(vec![("perf".into(), crate::Severity::Error)]);
    let source = "<script setup lang=\"ts\">\nimport { ref } from 'vue'\nconst state = ref({ count: 1 })\nconst outside = { ...state.value }\n</script>\n";
    let result = linter.lint_sfc(source, "Fixture.vue");
    let losses: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE_NO_REACTIVITY_LOSS)
        .collect();
    assert_eq!(losses.len(), 1);
    assert_eq!(losses[0].severity, crate::Severity::Error);
    assert_eq!(
        result.error_count,
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == crate::Severity::Error)
            .count()
    );
}
