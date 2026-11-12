//! no-floating-promises on template event handlers that only name or return a
//! call: the template owns the promise, so none of them floats.
use super::super::{RULE_NO_FLOATING_PROMISES, lint_sfc_with_corsa};
use super::assertion::assert_absent;
use super::corsa_available;
use crate::{LintPreset, Linter};

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
