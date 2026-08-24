#[path = "check_canon_fallthrough_attrs_cli/cases.rs"]
mod cases;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "check_canon_fallthrough_attrs_cli/support.rs"]
mod support;
use support::{
    assert_clean, assert_error_diagnostics, create_case, create_case_with_files,
    resolve_test_corsa_path, run_check_json,
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
fn check_fallthrough_attrs_keep_single_component_root_forwarding_open() {
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

    let report = run_check_json(&project_root, &corsa_path);
    assert_clean("fallthrough-component-root-open", &report);
    let _ = std::fs::remove_dir_all(project_root);
}
