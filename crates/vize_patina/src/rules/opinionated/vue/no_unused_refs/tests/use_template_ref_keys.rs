//! How `useTemplateRef` calls name the template refs they read.

use super::create_linter;

fn assert_unused(sfc: &str, expected: &[&str]) {
    let result = create_linter().lint_sfc(sfc, "test.vue");
    let messages: Vec<&str> = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(messages, expected);
}

#[test]
fn every_static_key_spelling_and_callee_form_keeps_its_ref() {
    let sfc = r#"<script setup lang="ts">
import * as Vue from 'vue'
import { onMounted, useTemplateRef, useTemplateRef as utr } from 'vue'

const KEY = 'viaConst'
const a = utr<HTMLInputElement>('viaAlias')
const b = Vue.useTemplateRef<HTMLInputElement>('viaNamespace')
const c = useTemplateRef<HTMLInputElement>(`viaTemplateLiteral`)
const d = useTemplateRef<HTMLInputElement>(KEY)
const e = useTemplateRef<HTMLInputElement>('viaAsConst' as const)
const f = useTemplateRef<HTMLInputElement>(('viaParens'))
const g = useTemplateRef<HTMLInputElement>('viaSatisfies' satisfies string)
onMounted(() => {
  console.log(a.value, b.value, c.value, d.value, e.value, f.value, g.value)
})
</script>

<template>
  <div>
    <input ref="viaAlias">
    <input ref="viaNamespace">
    <input ref="viaTemplateLiteral">
    <input ref="viaConst">
    <input ref="viaAsConst">
    <input ref="viaParens">
    <input ref="viaSatisfies">
  </div>
</template>"#;
    assert_unused(sfc, &[]);
}

#[test]
fn an_auto_imported_use_template_ref_keeps_its_ref() {
    let sfc = r#"<script setup lang="ts">
const el = useTemplateRef<HTMLInputElement>('field')
</script>
<template><input ref="field"></template>"#;
    assert_unused(sfc, &[]);
}

#[test]
fn a_runtime_key_makes_every_ref_reachable() {
    let sfc = r#"<script setup lang="ts">
import { onMounted, useTemplateRef } from 'vue'

function useFocus(key: string): void {
  const target = useTemplateRef<HTMLInputElement>(key)
  onMounted(() => {
    target.value?.focus()
  })
}
useFocus('field')
</script>

<template>
  <input ref="field">
</template>"#;
    assert_unused(sfc, &[]);
}

#[test]
fn a_static_key_does_not_keep_other_refs_alive() {
    let sfc = r#"<script setup lang="ts">
import { useTemplateRef as utr } from 'vue'
const a = utr('used')
</script>
<template><input ref="used"><input ref="dead"></template>"#;
    assert_unused(
        sfc,
        &["Template ref 'dead' is never referenced in <script>"],
    );
}

#[test]
fn a_handle_named_after_the_ref_does_not_read_it() {
    let sfc = r#"<script setup lang="ts">
import { onMounted, useTemplateRef } from 'vue'

const inputEl = useTemplateRef<HTMLInputElement>('input')
onMounted(() => {
  inputEl.value?.focus()
})
</script>

<template>
  <input ref="inputEl">
</template>"#;
    let result = create_linter().lint_sfc(sfc, "test.vue");
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].help.as_deref(),
        Some(
            "`inputEl` is useTemplateRef('input'): it reads ref=\"input\", and a binding name \
             never wires a template ref. Rename the attribute to ref=\"input\" or the key to \
             useTemplateRef('inputEl')."
        )
    );
}

#[test]
fn a_handle_whose_key_matches_its_name_reads_the_ref() {
    let sfc = r#"<script setup lang="ts">
import { useTemplateRef } from 'vue'
const input = useTemplateRef<HTMLInputElement>('input')
</script>
<template><input ref="input"></template>"#;
    assert_unused(sfc, &[]);
}
