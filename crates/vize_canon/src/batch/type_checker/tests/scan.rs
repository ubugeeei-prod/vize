use super::{BatchTypeChecker, unique_case_dir};

#[test]
fn test_batch_type_checker_scan() {
    let project_root = unique_case_dir("scan");
    let _ = std::fs::remove_dir_all(&project_root);
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let vue_content = r#"<template>
  <div>{{ message }}</div>
</template>

<script setup lang="ts">
const message = 'Hello'
</script>
"#;
    std::fs::write(src_dir.join("App.vue"), vue_content).unwrap();
    std::fs::write(src_dir.join("utils.ts"), "export const foo = 'bar';").unwrap();

    let mut checker = match BatchTypeChecker::new(&project_root) {
        Ok(checker) => checker,
        Err(_) => return,
    };

    checker.scan_project().unwrap();
    assert_eq!(checker.file_count(), 2);

    let _ = std::fs::remove_dir_all(&project_root);
}
