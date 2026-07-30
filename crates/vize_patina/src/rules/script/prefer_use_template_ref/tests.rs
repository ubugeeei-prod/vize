use crate::linter::{LintResult, Linter};
use crate::rules::script::PreferUseTemplateRef;
use crate::rules::script::ScriptLinter;

/// Lint a full SFC with only this rule enabled, exercising the engine path that
/// supplies the raw `<template>` source to the rule.
fn lint_sfc(sfc: &str) -> LintResult {
    Linter::new()
        .with_enabled_rules(Some(vec!["script/prefer-use-template-ref".into()]))
        .lint_sfc(sfc, "test.vue")
}

fn rule_names(result: &LintResult) -> Vec<&'static str> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name)
        .collect()
}

// --- Template pairing: the name must be bound as a template ref ------------

#[test]
fn reports_ref_null_paired_with_a_template_ref() {
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue'
const input = ref<HTMLInputElement | null>(null)
</script>

<template>
  <input ref="input" />
</template>
"#;
    let result = lint_sfc(sfc);
    assert_eq!(rule_names(&result), vec!["script/prefer-use-template-ref"]);
    insta::assert_debug_snapshot!(result.diagnostics);
}

#[test]
fn ignores_nullable_data_ref_the_template_never_binds() {
    // The false positive this rule shipped with: every `ref(null)` was reported
    // regardless of the template, so an ordinary nullable data ref was flagged
    // as a template ref.
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue'
const error = ref(null)
</script>

<template>
  <p>{{ error }}</p>
</template>
"#;
    assert_eq!(rule_names(&lint_sfc(sfc)), Vec::<&str>::new());
}

#[test]
fn ignores_nullable_data_ref_when_another_name_is_a_template_ref() {
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue'
const error = ref(null)
const root = ref(null)
</script>

<template>
  <div ref="root">{{ error }}</div>
</template>
"#;
    let result = lint_sfc(sfc);
    assert_eq!(rule_names(&result), vec!["script/prefer-use-template-ref"]);
    // Reported at the `root` declaration's call, not the `error` one.
    let root_call = sfc.rfind("ref(null)").expect("root declaration");
    assert_eq!(result.diagnostics[0].start as usize, root_call);
    assert_eq!(
        result.diagnostics[0].end as usize,
        root_call + "ref(null)".len()
    );
}

#[test]
fn ignores_dynamically_bound_ref_attributes() {
    // `:ref` binds an expression, not a template-ref name.
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue'
const root = ref(null)
</script>

<template>
  <div :ref="root" />
  <div v-bind:ref="root" />
</template>
"#;
    assert_eq!(rule_names(&lint_sfc(sfc)), Vec::<&str>::new());
}

#[test]
fn ignores_attributes_whose_name_merely_ends_in_ref() {
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue'
const root = ref(null)
</script>

<template>
  <a href="root">x</a>
  <div data-ref="root" />
</template>
"#;
    assert_eq!(rule_names(&lint_sfc(sfc)), Vec::<&str>::new());
}

// --- Initializer and callee coverage --------------------------------------

#[test]
fn reports_ref_without_an_initializer() {
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue'
const root = ref()
</script>

<template>
  <div ref="root" />
</template>
"#;
    assert_eq!(
        rule_names(&lint_sfc(sfc)),
        vec!["script/prefer-use-template-ref"]
    );
}

#[test]
fn reports_shallow_ref() {
    let sfc = r#"<script setup lang="ts">
import { shallowRef } from 'vue'
const root = shallowRef(null)
</script>

<template>
  <div ref="root" />
</template>
"#;
    assert_eq!(
        rule_names(&lint_sfc(sfc)),
        vec!["script/prefer-use-template-ref"]
    );
}

#[test]
fn ignores_use_template_ref() {
    let sfc = r#"<script setup lang="ts">
import { useTemplateRef } from 'vue'
const root = useTemplateRef('root')
</script>

<template>
  <div ref="root" />
</template>
"#;
    assert_eq!(rule_names(&lint_sfc(sfc)), Vec::<&str>::new());
}

#[test]
fn ignores_unrelated_ref_factories() {
    let sfc = r#"<script setup lang="ts">
import { toRef } from 'vue'
const root = toRef(state, 'root')
</script>

<template>
  <div ref="root" />
</template>
"#;
    assert_eq!(rule_names(&lint_sfc(sfc)), Vec::<&str>::new());
}

// --- Declaration position -------------------------------------------------

#[test]
fn reports_options_api_setup_declaration() {
    let sfc = r#"<script>
import { ref } from 'vue'
export default {
  setup() {
    const root = ref(null)
    return { root }
  },
}
</script>

<template>
  <div ref="root" />
</template>
"#;
    assert_eq!(
        rule_names(&lint_sfc(sfc)),
        vec!["script/prefer-use-template-ref"]
    );
}

#[test]
fn ignores_declaration_nested_in_a_callback() {
    // Only the script-setup program body and a `setup` body are template-ref
    // candidates; a ref created inside a callback is local state.
    let sfc = r#"<script setup lang="ts">
import { ref, watch } from 'vue'
watch(source, () => {
  const root = ref(null)
  use(root)
})
</script>

<template>
  <div ref="root" />
</template>
"#;
    assert_eq!(rule_names(&lint_sfc(sfc)), Vec::<&str>::new());
}

#[test]
fn reports_every_declarator_of_a_multi_declarator_statement() {
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue'
const count = ref(0), root = ref(null)
</script>

<template>
  <div ref="root">{{ count }}</div>
</template>
"#;
    assert_eq!(
        rule_names(&lint_sfc(sfc)),
        vec!["script/prefer-use-template-ref"]
    );
}

// --- No template to pair against ------------------------------------------

#[test]
fn reports_nothing_for_an_sfc_without_a_template() {
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue'
const root = ref(null)
</script>
"#;
    assert_eq!(rule_names(&lint_sfc(sfc)), Vec::<&str>::new());
}

#[test]
fn reports_nothing_for_a_standalone_script() {
    let mut linter = ScriptLinter::new();
    linter.add_rule(Box::new(PreferUseTemplateRef));
    let result = linter.lint("const root = ref(null)", 0);
    assert_eq!(result.warning_count, 0);
}
