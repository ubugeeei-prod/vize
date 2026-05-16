use super::{
    has_active_type_aware_rules, lint_sfc_with_corsa, RULE_NO_FLOATING_PROMISES,
    RULE_NO_REACTIVITY_LOSS, RULE_NO_UNSAFE_TEMPLATE_BINDING, RULE_REQUIRE_TYPED_EMITS,
    RULE_REQUIRE_TYPED_PROPS,
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
fn no_floating_promises_reports_control_flow_calls() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function loadData(): Promise<number> {
  return 1
}

const enabled = true
if (enabled) {
  loadData()
}
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn no_floating_promises_reports_finally_only_calls() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function loadData(): Promise<number> {
  return 1
}
function cleanup() {}

loadData().finally(cleanup)
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn handled_finally_chains_are_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function loadData(): Promise<number> {
  return 1
}
function cleanup() {}
function report(error: unknown) {
  console.error(error)
}

loadData().catch(report).finally(cleanup)
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn then_without_rejection_handler_reports_floating_promise() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function loadData(): Promise<number> {
  return 1
}

loadData().then((value) => console.log(value))
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn then_with_rejection_handler_is_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function loadData(): Promise<number> {
  return 1
}
function report(error: unknown) {
  console.error(error)
}

loadData().then((value) => console.log(value), report)
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn catch_without_handler_reports_floating_promise() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function loadData(): Promise<number> {
  return 1
}

loadData().catch()
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn no_floating_promises_reports_template_event_calls() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function save(): Promise<void> {}
</script>

<template>
  <button @click="save()">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result.diagnostics.iter().any(|diag| {
        diag.rule_name == RULE_NO_FLOATING_PROMISES
            && diag.message.contains("Template event handler")
    }));
}

#[test]
fn no_floating_promises_reports_bare_template_event_handlers() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function save(): Promise<void> {}
</script>

<template>
  <button @click="save">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result.diagnostics.iter().any(|diag| {
        diag.rule_name == RULE_NO_FLOATING_PROMISES
            && diag.message.contains("Template event handler")
    }));
}

#[test]
fn no_floating_promises_reports_member_template_event_handlers() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const actions = {
  async save(): Promise<void> {}
}
</script>

<template>
  <button @click="actions.save">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result.diagnostics.iter().any(|diag| {
        diag.rule_name == RULE_NO_FLOATING_PROMISES
            && diag.message.contains("Template event handler")
    }));
}

#[test]
fn no_floating_promises_ignores_sync_member_template_event_handlers() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const actions = {
  save(): void {}
}
</script>

<template>
  <button @click="actions.save">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn no_floating_promises_reports_optional_member_template_event_handlers() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
type Actions = {
  save(): Promise<void>
}
const actions: Actions | undefined = {
  async save(): Promise<void> {}
}
</script>

<template>
  <button @click="actions?.save">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result.diagnostics.iter().any(|diag| {
        diag.rule_name == RULE_NO_FLOATING_PROMISES
            && diag.message.contains("Template event handler")
    }));
}

#[test]
fn no_floating_promises_reports_computed_member_template_event_handlers() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const method = 'save' as const
const actions = {
  async save(): Promise<void> {}
}
</script>

<template>
  <button @click="actions[method]">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result.diagnostics.iter().any(|diag| {
        diag.rule_name == RULE_NO_FLOATING_PROMISES
            && diag.message.contains("Template event handler")
    }));
}

#[test]
fn no_floating_promises_ignores_sync_computed_member_template_event_handlers() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const method = 'save' as const
const actions = {
  save(): void {}
}
</script>

<template>
  <button @click="actions[method]">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn no_floating_promises_reports_optional_computed_member_template_event_handlers() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
type Actions = {
  save(): Promise<void>
}
const method = 'save' as const
const actions: Actions | undefined = {
  async save(): Promise<void> {}
}
</script>

<template>
  <button @click="actions?.[method]">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result.diagnostics.iter().any(|diag| {
        diag.rule_name == RULE_NO_FLOATING_PROMISES
            && diag.message.contains("Template event handler")
    }));
}

#[test]
fn no_floating_promises_reports_template_interpolations() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function loadLabel(): Promise<string> {
  return 'ready'
}
</script>

<template>
  <p>{{ loadLabel() }}</p>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result.diagnostics.iter().any(|diag| {
        diag.rule_name == RULE_NO_FLOATING_PROMISES
            && diag.message.contains("Template interpolation")
    }));
}

#[test]
fn no_floating_promises_reports_nested_template_event_calls() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const enabled = true
async function save(): Promise<void> {}
</script>

<template>
  <button @click="enabled && save()">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result.diagnostics.iter().any(|diag| {
        diag.rule_name == RULE_NO_FLOATING_PROMISES
            && diag.message.contains("Template event handler")
    }));
}

#[test]
fn no_floating_promises_reports_template_event_statement_blocks() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const enabled = true
async function save(): Promise<void> {}
</script>

<template>
  <button @click="if (enabled) { save() }">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result.diagnostics.iter().any(|diag| {
        diag.rule_name == RULE_NO_FLOATING_PROMISES
            && diag.message.contains("Template event handler")
    }));
}

#[test]
fn no_floating_promises_reports_nested_template_interpolations() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const enabled = true
async function loadLabel(): Promise<string> {
  return 'ready'
}
</script>

<template>
  <p>{{ enabled ? loadLabel() : 'idle' }}</p>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result.diagnostics.iter().any(|diag| {
        diag.rule_name == RULE_NO_FLOATING_PROMISES
            && diag.message.contains("Template interpolation")
    }));
}

#[test]
fn no_floating_promises_reports_template_finally_only_calls() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function save(): Promise<void> {}
function cleanup() {}
</script>

<template>
  <button @click="save().finally(cleanup)">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn handled_nested_template_promises_are_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const enabled = true
async function save(): Promise<void> {}
function report(error: unknown) {
  console.error(error)
}
</script>

<template>
  <button @click="enabled && save().catch(report)">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result
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
fn no_unsafe_template_binding_reports_event_statement_calls_after_prefix() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const safe = () => {}
const anyHandler: any = () => {}
</script>

<template>
  <button @click="safe(); anyHandler()">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "TypeAwareFixture.vue");

    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING));
}

#[test]
fn no_unsafe_template_binding_reports_computed_member_template_bindings() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const key = 'label' as const
const payload: Record<string, any> = { label: 'unsafe' }
</script>

<template>
  <div>{{ payload[key] }}</div>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "TypeAwareFixture.vue");

    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING));
}

#[test]
fn typed_computed_template_members_are_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const key = 'label' as const
const payload: { label: string } = { label: 'safe' }
</script>

<template>
  <div :title="payload[key]" />
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "TypeAwareFixture.vue");

    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING));
}

#[test]
fn no_reactivity_loss_tracks_props_calls_and_getter_returns() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const { count } = defineProps<{ count: number }>()

const ctx = useMyComposable(count)

const ctx2 = useMyComposable(() => count)
const a = ctx2.count()
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    let messages = result
        .diagnostics
        .iter()
        .filter(|diag| diag.rule_name == RULE_NO_REACTIVITY_LOSS)
        .map(|diag| diag.message.as_str())
        .collect::<Vec<_>>();

    assert!(messages
        .iter()
        .any(|message| message.contains("useMyComposable")));
    assert!(messages
        .iter()
        .any(|message| message.contains("ctx2.count()")));
}

#[test]
fn no_reactivity_loss_allows_direct_define_props_destructure() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const { count } = defineProps<{ count: number }>()
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_REACTIVITY_LOSS));
}

#[test]
fn no_reactivity_loss_uses_type_probe_to_keep_ref_typed_props() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
import type { Ref } from 'vue'

const props = defineProps<{ count: Ref<number> }>()
const count = props.count
const alias = count
useMyComposable(alias)
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_REACTIVITY_LOSS));
}

#[test]
fn no_reactivity_loss_tracks_plain_alias_chains() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const { count } = defineProps<{ count: number }>()

const alias = count
const second = alias
let assigned: number
assigned = second

useMyComposable(second)
useMyComposable(assigned)

const ctx = useMyComposable(() => second)
const a = ctx.second()
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    let messages = result
        .diagnostics
        .iter()
        .filter(|diag| diag.rule_name == RULE_NO_REACTIVITY_LOSS)
        .map(|diag| diag.message.as_str())
        .collect::<Vec<_>>();

    assert!(messages
        .iter()
        .any(|message| message.contains("plain snapshot 'count' to 'alias'")));
    assert!(messages
        .iter()
        .any(|message| message.contains("plain snapshot 'alias' to 'second'")));
    assert!(messages
        .iter()
        .any(|message| message.contains("plain snapshot 'second' to 'assigned'")));
    assert!(messages
        .iter()
        .any(|message| message.contains("Passing 'second'")));
    assert!(messages
        .iter()
        .any(|message| message.contains("Passing 'assigned'")));
    assert!(messages
        .iter()
        .any(|message| message.contains("ctx.second()")));
}

#[test]
fn no_reactivity_loss_reports_ref_value_and_reactive_member_snapshots() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
import { reactive, ref } from 'vue'

const countRef = ref(0)
const count = countRef.value

const state = reactive({ user: { name: 'Ada' } })
const user = state.user
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    let messages = result
        .diagnostics
        .iter()
        .filter(|diag| diag.rule_name == RULE_NO_REACTIVITY_LOSS)
        .map(|diag| diag.message.as_str())
        .collect::<Vec<_>>();

    assert!(messages
        .iter()
        .any(|message| message.contains("countRef.value")));
    assert!(messages
        .iter()
        .any(|message| message.contains("state.user")));
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
fn voided_template_promises_are_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function save(): Promise<void> {}
</script>

<template>
  <button @click="void save()">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn template_then_without_rejection_handler_reports_floating_promise() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function save(): Promise<void> {}
</script>

<template>
  <button @click="save().then(() => {})">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(result.diagnostics.iter().any(|diag| {
        diag.rule_name == RULE_NO_FLOATING_PROMISES
            && diag.message.contains("Template event handler")
    }));
}

#[test]
fn handled_template_promises_are_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function save(): Promise<void> {}
function report(error: unknown) {
  console.error(error)
}
</script>

<template>
  <button @click="save().catch(report)">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_FLOATING_PROMISES));
}

#[test]
fn handled_template_finally_chains_are_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function save(): Promise<void> {}
function cleanup() {}
function report(error: unknown) {
  console.error(error)
}
</script>

<template>
  <button @click="save().catch(report).finally(cleanup)">Save</button>
</template>"#;
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
fn typed_v_for_alias_template_bindings_are_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const items: Array<{ label: string }> = [{ label: 'safe' }]
</script>

<template>
  <p v-for="item in items" :key="item.label">{{ item.label }}</p>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert!(!result
        .diagnostics
        .iter()
        .any(|diag| diag.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING));
}

#[test]
fn typed_template_members_after_setup_object_literals_are_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const payload: { label: string } = { label: 'safe' }
const actions = {
  save(): void {}
}
</script>

<template>
  <button :title="payload.label" @click="actions.save">{{ payload.label }}</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "SafePanel.vue");
    assert!(!result.diagnostics.iter().any(|diag| matches!(
        diag.rule_name,
        RULE_NO_UNSAFE_TEMPLATE_BINDING | RULE_NO_FLOATING_PROMISES
    )));
}

#[test]
fn type_aware_diagnostics_snapshot() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
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
