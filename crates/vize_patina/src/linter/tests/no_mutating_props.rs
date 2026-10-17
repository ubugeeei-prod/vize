use super::Linter;

fn no_mutating_props_linter() -> Linter {
    Linter::new().with_enabled_rules(Some(vec!["vue/no-mutating-props".into()]))
}

#[test]
fn allows_local_ref_that_shadows_an_implicit_prop_binding() {
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{ enabled?: boolean }>()
const enabled = ref(props.enabled)
</script>

<template>
  <Switch v-model="enabled" />
</template>
"#;
    let result = no_mutating_props_linter().lint_sfc(sfc, "test.vue");

    assert!(
        result.diagnostics.is_empty(),
        "a local ref must shadow the implicit prop binding: {:?}",
        result.diagnostics
    );
}

#[test]
fn allows_define_model_binding() {
    let sfc = r#"<script setup lang="ts">
const modelValue = defineModel<string>()
</script>

<template>
  <input v-model="modelValue" />
</template>
"#;
    let result = no_mutating_props_linter().lint_sfc(sfc, "test.vue");

    assert!(
        result.diagnostics.is_empty(),
        "defineModel is a writable local ref: {:?}",
        result.diagnostics
    );
}

#[test]
fn class_decorator_props_keep_the_nested_v_model_mutation_diagnostic() {
    let source = include_str!(
        "../../../../../tests/_fixtures/differential/linter/reference-coverage-8849/card.vue.txt"
    );
    let result = no_mutating_props_linter().lint_sfc(source, "card.vue");
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    let diagnostic = &result.diagnostics[0];
    let start = source.find("notebook.notes").unwrap() as u32;
    assert_eq!((diagnostic.start, diagnostic.end), (start, start + 14));
    assert_eq!(diagnostic.severity, crate::Severity::Error);
    assert_eq!(diagnostic.rule_name, "vue/no-mutating-props");
    assert_eq!(
        diagnostic.message.as_str(),
        "Unexpected mutation of prop 'notebook.notes' via v-model"
    );

    let local = source.replace("@Prop({ type: Object, default: () => ({}) })", "");
    assert!(
        no_mutating_props_linter()
            .lint_sfc(&local, "local.vue")
            .diagnostics
            .is_empty()
    );
    let read_only = source.replace(
        "v-model=\"notebook.notes\"",
        ":model-value=\"notebook.notes\"",
    );
    assert!(
        no_mutating_props_linter()
            .lint_sfc(&read_only, "read-only.vue")
            .diagnostics
            .is_empty()
    );
}
