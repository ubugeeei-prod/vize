use super::{ORIGINAL, case, check, corsa_requirement, fixture, native_emit_oracle, write};
use std::path::PathBuf;

#[test]
fn strict_generated_template_context_uses_the_same_declared_emit_vector() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let case = case();
    fixture(case.path(), ORIGINAL);
    write(case.path(), "nuxt.config.ts", "export default {};\n");
    write(case.path(), ".nuxt/imports.d.ts", "export {};\n");
    native_emit_oracle(
        case.path(),
        &corsa,
        "const emit = defineEmits<{ click: []; change: [value: number] }>();",
        "emit('clik');\nemit('change', 'x');",
        concat!(
            "oracle.ts(3,6): error TS2345: Argument of type '\"clik\"' is not assignable to parameter of type '\"click\"'.\n",
            "oracle.ts(4,16): error TS2345: Argument of type 'string' is not assignable to parameter of type 'number'.\n"
        ),
    );
    check(
        case.path(),
        &corsa,
        &[
            "error:9:39 [TS2345] Argument of type '\"clik\"' is not assignable to parameter of type '\"click\"'.",
            "error:10:49 [TS2345] Argument of type 'string' is not assignable to parameter of type 'number'.",
        ],
    );
}

const MODEL_ERROR: &str = concat!(
    "No overload matches this call.\n",
    "The last overload gave the following error.\n",
    "Argument of type 'number' is not assignable to parameter of type 'string'."
);
const NATIVE_MODEL_ERROR: &str = concat!(
    "No overload matches this call.\n",
    "  The last overload gave the following error.\n",
    "    Argument of type 'number' is not assignable to parameter of type 'string'."
);

#[test]
fn declared_emit_and_model_update_keep_both_typed_event_contracts() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let case = case();
    fixture(
        case.path(),
        r#"<script setup lang="ts">
defineEmits<{ change: [value: number] }>();
defineModel<string>({ required: true });
</script>
<template>
  <button @click="$emit('update:modelValue', 'x'); $emit('change', 1)">ok</button>
  <button @click="$emit('update:modelValue', 1)">bad</button>
</template>
"#,
    );
    native_emit_oracle(
        case.path(),
        &corsa,
        "const model = defineModel<string>({ required: true }); const emit = defineEmits<{ change: [value: number]; 'update:modelValue': [value: typeof model.value] }>();",
        "emit('update:modelValue', 'x');\nemit('update:modelValue', 1);",
        &format!("oracle.ts(4,27): error TS2769: {NATIVE_MODEL_ERROR}\n"),
    );
    check(
        case.path(),
        &corsa,
        &[&format!("error:7:46 [TS2769] {MODEL_ERROR}")],
    );
}

#[test]
fn generic_setup_emit_preserves_its_local_type_parameter() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let case = case();
    fixture(
        case.path(),
        r#"<script setup lang="ts" generic="T extends number">
defineProps<{ value: T }>();
defineEmits<{ change: [value: T] }>();
</script>
<template>
  <button @click="$emit('change', value)">ok</button>
</template>
"#,
    );
    native_emit_oracle(
        case.path(),
        &corsa,
        "function setup<T extends number>(value: T) { const emit = defineEmits<{ change: [value: T] }>(); emit('change', value); }",
        "",
        "",
    );
    check(case.path(), &corsa, &[]);
}
