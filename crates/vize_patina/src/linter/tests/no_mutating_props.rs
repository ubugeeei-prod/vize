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
