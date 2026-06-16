use std::path::{Path, PathBuf};

use super::{create_project_case, relative_path, resolve_test_tsgo_binary};
use crate::batch::Diagnostic;
use crate::batch::TypeChecker;
use crate::batch::type_checker::{BatchTypeChecker, BatchTypeCheckerOptions, TypeCheckResult};

#[test]
fn test_type_check_result() {
    let mut result = TypeCheckResult::default();
    assert!(!result.has_errors());
    assert_eq!(result.error_count(), 0);

    result.diagnostics.push(Diagnostic {
        file: PathBuf::from("test.vue"),
        line: 0,
        column: 0,
        message: "error".into(),
        code: Some(2304),
        severity: 1,
        block_type: None,
    });

    assert!(result.has_errors());
    assert_eq!(result.error_count(), 1);
}

#[test]
fn batch_type_checker_reports_runtime_emit_object_instance_props_recursion() {
    let Some(corsa_path) = resolve_test_tsgo_binary().or_else(resolve_workspace_tsgo_wrapper)
    else {
        return;
    };
    let project_root = create_project_case(
        "runtime-emit-object-instance-props-recursion",
        &[
            (
                "src/Test.vue",
                r#"<template></template>
<script setup lang="ts">
const emit = defineEmits({
  test: (value1: string, value2: number) => {
    console.log(value1, value2);
  },
});
</script>
"#,
            ),
            (
                "src/test.ts",
                r#"import Test from "./Test.vue";

type TestProps = InstanceType<typeof Test>["$props"];
void (null as unknown as TestProps);
"#,
            ),
        ],
    );

    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        &project_root,
        BatchTypeCheckerOptions::default(),
        Some(corsa_path.as_path()),
    )
    .expect("batch checker should initialize with explicit tsgo");
    checker.scan_project().expect("project should scan");
    let result = checker.check_project().expect("project should check");

    let found = result.diagnostics.iter().any(|diagnostic| {
        relative_path(project_root.as_path(), &diagnostic.file) == "src/test.ts"
            && diagnostic.code == Some(2589)
            && diagnostic
                .message
                .contains("Type instantiation is excessively deep")
    });
    assert!(
        found,
        "expected TS2589 in the TS consumer, got: {:?}",
        result.diagnostics
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

fn resolve_workspace_tsgo_wrapper() -> Option<PathBuf> {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)?;
    let wrapper = workspace_root.join("node_modules/.bin/tsgo");
    wrapper.exists().then_some(wrapper)
}
