use super::{LintPreset, Linter};
use crate::rules::script::EventNameCasing;

#[test]
fn opinionated_uses_kebab_case_for_custom_events() {
    let camel = r#"<script setup lang="ts">
const emit = defineEmits<{ saveItem: [] }>()
emit('saveItem')
</script>"#;
    let kebab = r#"<script setup lang="ts">
const emit = defineEmits<{ "save-item": [] }>()
emit('save-item')
</script>"#;

    let camel_result = Linter::with_preset(LintPreset::Opinionated).lint_sfc(camel, "App.vue");
    assert!(
        camel_result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/custom-event-name-casing")
    );

    let kebab_result = Linter::with_preset(LintPreset::Opinionated).lint_sfc(kebab, "App.vue");
    assert!(
        !kebab_result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/custom-event-name-casing")
    );
}

#[test]
fn explicit_custom_event_casing_overrides_opinionated() {
    let source = r#"<script setup lang="ts">
const emit = defineEmits<{ saveItem: [] }>()
emit('saveItem')
</script>"#;
    let result = Linter::with_preset(LintPreset::Opinionated)
        .with_custom_event_name_casing(EventNameCasing::CamelCase)
        .lint_sfc(source, "App.vue");

    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "script/custom-event-name-casing")
    );
}

#[test]
fn consistent_props_emits_key_quotes_is_opinionated_only() {
    let source = r#"<script setup lang="ts">
defineProps<{ "aria-label": string; disabled: boolean }>()
</script>"#;

    let opinionated = Linter::with_preset(LintPreset::Opinionated).lint_sfc(source, "App.vue");
    assert!(
        opinionated.diagnostics.iter().any(|diagnostic| {
            diagnostic.rule_name == "script/consistent-props-emits-key-quotes"
        })
    );

    let nuxt = Linter::with_preset(LintPreset::Nuxt).lint_sfc(source, "App.vue");
    assert!(
        !nuxt.diagnostics.iter().any(|diagnostic| {
            diagnostic.rule_name == "script/consistent-props-emits-key-quotes"
        })
    );
}
