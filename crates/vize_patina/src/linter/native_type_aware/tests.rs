pub(in crate::linter::native_type_aware) mod assertion;
mod severity;

mod opt_in;
mod options_api;
use super::{
    RULE_NO_FLOATING_PROMISES, RULE_NO_REACTIVITY_LOSS, RULE_NO_UNSAFE_TEMPLATE_BINDING,
    RULE_REQUIRE_TYPED_EMITS, RULE_REQUIRE_TYPED_PROPS, lint_sfc_with_corsa,
};
use crate::{LintPreset, Linter};

use assertion::{
    assert_absent, assert_absent_note, assert_absent_rules, assert_reports, assert_reports_message,
    assert_reports_note, failure_context,
};

pub(super) fn corsa_available() -> bool {
    let mut session =
        match super::CorsaTypeAwareSession::new_with_corsa_path("Component.vue", None, None) {
            Ok(session) => session,
            Err(_) => return false,
        };
    if session
        .open_virtual_project("const value = 1;\n", "Component.vue")
        .is_err()
    {
        session.close();
        return false;
    }
    session.close();
    true
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
    assert_reports(&result, RULE_REQUIRE_TYPED_PROPS);
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
    assert_reports(&result, RULE_REQUIRE_TYPED_EMITS);
}

#[test]
fn require_typed_emits_accepts_quoted_colon_key_and_reports_runtime_array() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let typed_source = r#"<script setup lang="ts">
defineEmits<{ "update:open": [value: boolean] }>()
</script>"#;
    let typed_result = lint_sfc_with_corsa(&linter, typed_source, "TypedDialog.vue");
    assert_absent_note(
        &typed_result,
        RULE_REQUIRE_TYPED_EMITS,
        "quoted colon emit keys should be treated as typed",
    );

    let runtime_source = r#"<script setup lang="ts">
defineEmits(["update:open"])
</script>"#;
    let runtime_result = lint_sfc_with_corsa(&linter, runtime_source, "RuntimeDialog.vue");
    assert_reports_note(
        &runtime_result,
        RULE_REQUIRE_TYPED_EMITS,
        "runtime-only emits should still be reported",
    );
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
    assert_reports(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_reports(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_reports(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_reports(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_reports(&result, RULE_NO_FLOATING_PROMISES);
}

#[test]
fn no_floating_promises_ignores_returned_template_event_calls() {
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
}

#[test]
fn no_floating_promises_ignores_returned_inline_arrow_calls() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function save(): Promise<void> {}
</script>
<template><button @click="() => save()">Save</button></template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
}

#[test]
fn malformed_template_event_expressions_skip_type_aware_noise() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
async function save(): Promise<void> {}
</script>

<template>
  <button @click="save(">Save</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert_absent_note(
        &result,
        RULE_NO_FLOATING_PROMISES,
        "malformed template event should not emit floating-promise noise",
    );
}

#[test]
fn no_floating_promises_ignores_bare_template_event_handlers() {
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
}

#[test]
fn no_floating_promises_ignores_member_template_event_handlers() {
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
}

#[test]
fn no_floating_promises_ignores_optional_member_template_event_handlers() {
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
}

#[test]
fn no_floating_promises_ignores_computed_member_template_event_handlers() {
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
}

#[test]
fn no_floating_promises_ignores_optional_computed_member_template_event_handlers() {
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_reports_message(&result, RULE_NO_FLOATING_PROMISES, "Template interpolation");
}

#[test]
fn no_floating_promises_ignores_returned_nested_template_event_calls() {
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_reports_message(&result, RULE_NO_FLOATING_PROMISES, "Template event handler");
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
    assert_reports_message(&result, RULE_NO_FLOATING_PROMISES, "Template interpolation");
}

#[test]
fn no_floating_promises_ignores_returned_template_finally_only_calls() {
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_reports(&result, RULE_NO_UNSAFE_TEMPLATE_BINDING);
}

#[test]
fn typed_shorthand_binding_and_bare_event_handler_are_safe() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const name: string = 'ready'
function onClick(event: MouseEvent): void { void event }
const unsafeValue: any = 'unsafe'
</script>
<template>
  <button :name @click="onClick">{{ unsafeValue }}</button>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "ShorthandAndHandler.vue");
    let unsafe_start = source.find("unsafeValue }}</button>").unwrap() as u32;
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diag| diag.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING)
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "unexpected unsafe bindings\n{}",
        failure_context(&result)
    );
    assert_eq!(warnings[0].start, unsafe_start);
}

#[test]
fn typed_v_for_sources_use_the_inferred_iterable_type() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
type Item = { readonly name: string }
const items: readonly Item[] = [{ name: 'safe' }]
function getNames(): string[] { return ['safe'] }
</script>
<template>
  <span v-for="item in items as readonly Item[]" :key="item.name" />
  <span v-for="name in getNames()" :key="name" />
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "TypedForSources.vue");
    assert_absent_note(
        &result,
        RULE_NO_UNSAFE_TEMPLATE_BINDING,
        "typed v-for sources should stay safe",
    );
}

#[test]
fn multiline_v_for_sources_use_the_inferred_iterable_type() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const pages: string[][] = [['a', 'b'], ['c']]
const current = 0
function pick(index: number): string[] { return pages[index] ?? [] }
</script>
<template>
  <li v-for="item in pages[
    current
  ]" :key="item">{{ item }}</li>
  <li v-for="item in pick(
    current
  )" :key="item">{{ item }}</li>
  <li v-for="item in
    pick(current)" :key="item">{{ item }}</li>
  <li v-for="item in pick(current)" :key="item">{{ item }}</li>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "MultilineForSources.vue");
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING)
        .collect();
    assert!(
        warnings.is_empty(),
        "typed multiline v-for sources should stay safe\n{}",
        failure_context(&result)
    );
}

#[test]
fn unsafe_v_for_source_still_reports() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const items: any = ['unsafe']
</script>
<template><span v-for="item in items" /></template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "UnsafeForSource.vue");
    assert_reports_note(
        &result,
        RULE_NO_UNSAFE_TEMPLATE_BINDING,
        "an explicit any iterable must remain unsafe",
    );
}

#[test]
fn slot_spread_calls_use_return_type_without_hiding_any() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
defineSlots<{ default(props: { label: string }): unknown }>()
function typed(): { label: string } { return { label: 'safe' } }
function unsafe(): any { return { label: 'unsafe' } }
</script>
<template>
  <slot v-bind="typed()" />
  <slot v-bind="unsafe()" />
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "SlotSpreadCalls.vue");
    let warnings: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|diag| diag.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING)
        .collect();
    assert_eq!(
        warnings.len(),
        1,
        "unexpected slot spread warnings\n{}",
        failure_context(&result)
    );
    assert_eq!(
        warnings[0].start,
        source.find("unsafe()\" />").unwrap() as u32
    );
}

#[test]
fn relative_imports_keep_template_bindings_typed_across_source_directories() {
    use std::path::PathBuf;
    use vize_carton::corsa_resolver::{CorsaResolveRequest, resolve_corsa_executable};

    if !corsa_available() {
        return;
    }

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap()
        .to_path_buf();
    let Ok(corsa_path) = resolve_corsa_executable(CorsaResolveRequest {
        explicit_path: None,
        project_root: Some(&repo_root),
    }) else {
        return;
    };
    let package = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/vize-tests")
        .join(format!("patina-imports-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&package);
    let first = package.join("src/first");
    let second = package.join("src/second");
    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();
    std::fs::write(package.join("package.json"), "{}\n").unwrap();
    std::fs::write(
        first.join("model.ts"),
        "export const safe = { message: 'typed' };\nexport const unsafe: any = { message: 'unsafe' };\n",
    )
    .unwrap();
    std::fs::write(
        second.join("model.ts"),
        "export const safe = { message: 'also typed' };\n",
    )
    .unwrap();

    let linter = Linter::with_preset(LintPreset::Opinionated).with_corsa_path(Some(corsa_path));
    let first_source = "<script setup lang=\"ts\">\nimport { safe, unsafe } from './model.ts'\n</script>\n<template><p>{{ safe.message }}</p><p>{{ unsafe.message }}</p></template>";
    let first_result = lint_sfc_with_corsa(
        &linter,
        first_source,
        first.join("Panel.vue").to_str().unwrap(),
    );
    let safe_offset = first_source.find("safe.message").unwrap() as u32;
    let unsafe_offset = first_source.find("unsafe.message").unwrap() as u32;
    assert!(
        first_result.diagnostics.iter().any(|diag| {
            diag.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING
                && diag.start >= unsafe_offset
                && diag.end <= unsafe_offset + "unsafe.message".len() as u32
        }),
        "explicit any should remain unsafe\n{}",
        failure_context(&first_result)
    );
    assert!(
        !first_result.diagnostics.iter().any(|diag| {
            diag.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING
                && diag.start >= safe_offset
                && diag.start < unsafe_offset
        }),
        "a typed relative import must not become any\n{}",
        failure_context(&first_result)
    );

    let second_source = "<script setup lang=\"ts\">\nimport { safe } from './model.ts'\n</script>\n<template><p>{{ safe.message }}</p></template>";
    let second_result = lint_sfc_with_corsa(
        &linter,
        second_source,
        second.join("Panel.vue").to_str().unwrap(),
    );
    assert!(
        !second_result
            .diagnostics
            .iter()
            .any(|diag| diag.rule_name == RULE_NO_UNSAFE_TEMPLATE_BINDING),
        "moving the virtual file must preserve the second import\n{}",
        failure_context(&second_result)
    );
    drop(linter);
    std::fs::remove_dir_all(package).unwrap();
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

    assert_reports(&result, RULE_NO_UNSAFE_TEMPLATE_BINDING);
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

    assert_reports(&result, RULE_NO_UNSAFE_TEMPLATE_BINDING);
}

#[test]
fn malformed_template_interpolations_skip_type_aware_noise() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const payload: any = { label: 'unsafe' }
</script>

<template>
  <div>{{ payload. }}</div>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "TypeAwareFixture.vue");
    assert_absent_note(
        &result,
        RULE_NO_UNSAFE_TEMPLATE_BINDING,
        "malformed interpolation should not emit unsafe-binding noise",
    );
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

    assert_absent(&result, RULE_NO_UNSAFE_TEMPLATE_BINDING);
}

#[test]
fn no_reactivity_loss_tracks_props_composable_calls_and_getter_returns() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const { count } = defineProps<{ count: number }>()

useMyComposable(count)

const ctx2 = useMyComposable(() => count)
const a = ctx2.count()
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert_reports_message(&result, RULE_NO_REACTIVITY_LOSS, "useMyComposable");
    assert_reports_message(&result, RULE_NO_REACTIVITY_LOSS, "ctx2.count()");
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
    assert_absent(&result, RULE_NO_REACTIVITY_LOSS);
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
    assert_absent(&result, RULE_NO_REACTIVITY_LOSS);
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
assigned++

const ctx = useMyComposable(() => second)
const a = ctx.second()
</script>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert_reports_message(
        &result,
        RULE_NO_REACTIVITY_LOSS,
        "plain snapshot 'count' to 'alias'",
    );
    assert_reports_message(
        &result,
        RULE_NO_REACTIVITY_LOSS,
        "plain snapshot 'alias' to 'second'",
    );
    assert_reports_message(
        &result,
        RULE_NO_REACTIVITY_LOSS,
        "plain snapshot 'second' to 'assigned'",
    );
    assert_reports_message(&result, RULE_NO_REACTIVITY_LOSS, "Passing 'second'");
    assert_reports_message(&result, RULE_NO_REACTIVITY_LOSS, "Passing 'assigned'");
    assert_reports_message(&result, RULE_NO_REACTIVITY_LOSS, "ctx.second()");
    assert_reports_message(&result, RULE_NO_REACTIVITY_LOSS, "Mutating 'assigned'");
    assert_reports_message(&result, RULE_NO_REACTIVITY_LOSS, "Passing 'second'");
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
    assert_reports_message(&result, RULE_NO_REACTIVITY_LOSS, "countRef.value");
    assert_reports_message(&result, RULE_NO_REACTIVITY_LOSS, "state.user");
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
}

#[test]
fn returned_template_then_without_rejection_handler_is_observed() {
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_absent(&result, RULE_NO_FLOATING_PROMISES);
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
    assert_absent_rules(
        &result,
        &[RULE_REQUIRE_TYPED_PROPS, RULE_REQUIRE_TYPED_EMITS],
    );
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
    assert_absent(&result, RULE_NO_UNSAFE_TEMPLATE_BINDING);
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
    assert_absent(&result, RULE_NO_UNSAFE_TEMPLATE_BINDING);
}

#[test]
fn typed_destructured_v_for_template_bindings_are_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const items: Array<{ id: number; meta: { label: string } }> = [
  { id: 1, meta: { label: 'safe' } },
]
</script>

<template>
  <p v-for="{ id, meta: { label } } in items" :key="id">{{ label.toUpperCase() }}</p>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "Component.vue");
    assert_absent_note(
        &result,
        RULE_NO_UNSAFE_TEMPLATE_BINDING,
        "destructured v-for bindings should stay typed",
    );
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
    assert_absent_rules(
        &result,
        &[RULE_NO_UNSAFE_TEMPLATE_BINDING, RULE_NO_FLOATING_PROMISES],
    );
}

#[test]
fn narrowed_template_bindings_inside_v_if_are_ignored() {
    if !corsa_available() {
        return;
    }

    let linter = Linter::with_preset(LintPreset::Opinionated);
    let source = r#"<script setup lang="ts">
const payload: unknown = 'safe'
</script>

<template>
  <p v-if="typeof payload === 'string'">{{ payload.toUpperCase() }}</p>
</template>"#;
    let result = lint_sfc_with_corsa(&linter, source, "NarrowedTemplate.vue");

    assert_absent_note(
        &result,
        RULE_NO_UNSAFE_TEMPLATE_BINDING,
        "v-if narrowing should keep template binding safe",
    );
}
