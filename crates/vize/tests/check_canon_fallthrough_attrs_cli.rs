mod cases;
use crate::support::corsa_requirement;
mod support;
use support::{
    assert_clean, assert_error_diagnostics, create_case, create_case_with_files,
    resolve_test_corsa_path, run_check_json, write_vue_tsc_default_tsconfig,
};
#[test]
fn check_fallthrough_attrs_follow_inherit_attrs_and_root_shape() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };

    let cases = cases::cases();

    for case in cases {
        let project_root = create_case(case.id, case.child, case.app);
        let report = run_check_json(&project_root, &corsa_path);
        if case.expected_diagnostics.is_empty() {
            assert_clean(case.id, &report);
        } else {
            assert_error_diagnostics(case.id, &report, case.expected_diagnostics);
        }
        let _ = std::fs::remove_dir_all(project_root);
    }
}

#[test]
fn check_single_component_root_forwarding_follows_fallthrough_attributes() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };

    let project_root = create_case_with_files(
        "fallthrough-component-root-open",
        r#"<script setup lang="ts">
import BaseInput from "./BaseInput.vue";

defineProps<{ title: string }>();
</script>
<template><BaseInput /></template>
"#,
        r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="ok" model-value="draft" data-test-id="base-input" w-48px /></template>
"#,
        &[(
            "BaseInput.vue",
            r#"<script setup lang="ts">
defineProps<{ modelValue?: string }>();
</script>
<template><input :value="modelValue" /></template>
"#,
        )],
    );

    // `strictTemplates` alone does not infer the root's attributes: the root
    // component's prop is unknown on `Child`, as vue-tsc reports it.
    let report = run_check_json(&project_root, &corsa_path);
    assert_error_diagnostics(
        "fallthrough-component-root-strict",
        &report,
        &[
            r#"error:4:29 [TS2353] Object literal may only specify known properties, and '"modelValue"' does not exist in type '$ComponentCheckProps<Props, $CheckTail>'. (source: <template><Child title="ok" model-value="draft" data-test-id="base-input" w-48px /></template>)"#,
        ],
    );
    // With `fallthroughAttributes`, the root component's props are `Child`'s.
    let tsconfig = project_root.join("tsconfig.json");
    let strict = std::fs::read_to_string(&tsconfig).unwrap();
    std::fs::write(
        &tsconfig,
        strict.replace(
            r#""strictTemplates": true"#,
            r#""strictTemplates": true, "fallthroughAttributes": true"#,
        ),
    )
    .unwrap();
    let report = run_check_json(&project_root, &corsa_path);
    assert_clean("fallthrough-component-root-open", &report);
    let _ = std::fs::remove_dir_all(project_root);
}

#[test]
fn unknown_extra_attrs_match_vue_tsc_without_strict_templates() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };

    let project_root = create_case(
        "unknown-extras-vue-tsc-default",
        r#"<script setup lang="ts">
defineProps<{ height?: number; start?: string; end?: string }>();
</script>
<template><svg /></template>
"#,
        r#"<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :gradient="false" disable /></template>
"#,
    );
    write_vue_tsc_default_tsconfig(&project_root);
    let report = run_check_json(&project_root, &corsa_path);
    assert_clean("unknown-extras-vue-tsc-default", &report);
    let _ = std::fs::remove_dir_all(project_root);
}
