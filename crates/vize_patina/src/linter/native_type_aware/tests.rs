use super::{
    has_active_type_aware_rules, lint_sfc_with_corsa, RULE_NO_FLOATING_PROMISES,
    RULE_NO_UNSAFE_TEMPLATE_BINDING, RULE_REQUIRE_TYPED_EMITS, RULE_REQUIRE_TYPED_PROPS,
};
use crate::{LintPreset, Linter};

fn corsa_available() -> bool {
    let mut session = match super::CorsaTypeAwareSession::new("Component.vue") {
        Ok(session) => session,
        Err(_) => return false,
    };
    if session.open_virtual_project("const value = 1;\n").is_err() {
        session.close();
        return false;
    }
    session.close();
    true
}

#[test]
fn opinionated_preset_enables_native_type_aware_rules() {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    assert!(has_active_type_aware_rules(&linter));
}

#[test]
fn require_typed_props_uses_corsa() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
defineProps(['msg', 'count'])
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_REQUIRE_TYPED_PROPS));
}

#[test]
fn require_typed_emits_uses_corsa() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
defineEmits(['save'])
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_REQUIRE_TYPED_EMITS));
}

#[test]
fn no_floating_promises_uses_corsa() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function loadData(): Promise<number> {
  return 1
}

loadData()
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn no_unsafe_template_binding_uses_corsa() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const payload: any = { label: 'unsafe' }
const anyHandler: any = () => {}
</script>

<template>
  <div>{{ payload.label }}</div>
  <button @click="anyHandler()">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "TypeAwareFixture.vue");
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING));
}

#[test]
fn voided_promises_are_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function loadData(): Promise<number> {
  return 1
}

void loadData()
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn runtime_validators_are_treated_as_typed() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
defineProps({
  msg: { type: String, required: true },
  count: { type: Number, default: 0 },
})

defineEmits({
  save: (value: number) => typeof value === 'number',
})
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result.diagnostics.iter().any(|diag| matches!(
        diag.rule_name,
        RULE_REQUIRE_TYPED_PROPS | RULE_REQUIRE_TYPED_EMITS
    )));
}

#[test]
fn typed_template_bindings_are_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const payload = { label: 'safe' }
const onSave = () => {}
</script>

<template>
  <div>{{ payload.label }}</div>
  <button @click="onSave">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING));
}

#[test]
fn type_aware_diagnostics_snapshot() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
defineProps(['msg'])
defineEmits(['save'])
const payload: any = { label: 'unsafe' }
const anyHandler: any = () => {}

async function loadData(): Promise<number> {
  return 1
}

loadData()
</script>

<template>
  <div>{{ payload.label }}</div>
  <button @click="anyHandler()">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "TypeAwareFixture.vue");
    let diagnostics = result
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
    insta::assert_debug_snapshot!(diagnostics);
}
