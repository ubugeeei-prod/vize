//! A parser error in one TypeScript file must not drop per-file SFC modules.

use std::path::PathBuf;

use super::super::{create_project_case, resolve_test_tsgo_binary};
use crate::batch::TypeChecker;
use crate::{BatchTypeChecker, BatchTypeCheckerOptions};

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "exclude": ["node_modules"]
}
"#;

const WIDGET: &str = r#"<script setup lang="ts">
export interface Item { id: number }
defineProps<{ label: string }>()
</script>
<template>{{ label }}</template>
"#;

const USE: &str = r#"import Widget from "./Widget.vue";
export type { Item } from "./Widget.vue";
export const n = Widget ? 1 : 0;
"#;

const BROKEN: &str = "export const bad: = 1;\n";

const SHIM: &str = r#"declare module "*.vue" {
  import Vue from "vue";
  export default Vue;
}
"#;

#[test]
fn issue_10353_syntax_error_keeps_sfc_module_types() {
    let Some(native) = resolve_test_tsgo_binary() else {
        return;
    };
    let project_root = create_project_case(
        "sfc-syntax-error-module-10353",
        &[
            ("vue-shim.d.ts", SHIM),
            ("src/Widget.vue", WIDGET),
            ("src/use.ts", USE),
            ("src/broken.ts", BROKEN),
        ],
    );
    std::fs::write(project_root.join("tsconfig.json"), TSCONFIG).unwrap();
    let files = [
        project_root.join("vue-shim.d.ts"),
        project_root.join("src/Widget.vue"),
        project_root.join("src/use.ts"),
        project_root.join("src/broken.ts"),
    ];
    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        &project_root,
        BatchTypeCheckerOptions::default(),
        Some(native.as_path()),
    )
    .unwrap();
    checker.set_server_count(Some(1));
    checker.set_checker_count(Some(1));
    checker.scan_paths(&files).unwrap();
    checker.set_diagnostic_paths(files.iter().map(PathBuf::as_path));

    let mirrored = checker.virtual_files();
    let widget = mirrored
        .iter()
        .find(|file| file.original_path.ends_with("Widget.vue"))
        .expect("Widget.vue must be materialized");
    let importer = mirrored
        .iter()
        .find(|file| file.original_path.ends_with("use.ts"))
        .expect("use.ts must be materialized");
    let broken = mirrored
        .iter()
        .find(|file| file.original_path.ends_with("broken.ts"))
        .expect("broken.ts must be materialized");
    assert!(
        widget.content.contains("export interface Item"),
        "SFC module dropped its interface export:\n{}",
        widget.content
    );
    assert!(
        !importer.content.contains("\"*.vue\"") && !importer.content.contains("'*.vue'"),
        "importer was redirected to the wildcard shim:\n{}",
        importer.content
    );
    assert!(
        importer.content.contains("Widget.vue.ts"),
        "importer must keep the per-file SFC module:\n{}",
        importer.content
    );
    assert!(
        broken.content.contains("export const bad: = 1"),
        "syntax error was rewritten away:\n{}",
        broken.content
    );
    let importer_content = importer.content.clone();
    let widget_content = widget.content.clone();
    drop(mirrored);

    let (virtual_root, config) = checker.native_project_paths_for_test();
    let result = checker
        .check_project()
        .expect("check must return diagnostics");
    let rendered = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            format!(
                "{}:{}:{} TS{:?} {}",
                diagnostic.file.display(),
                diagnostic.line,
                diagnostic.column,
                diagnostic.code,
                diagnostic.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let generated = std::fs::read_to_string(&config).unwrap_or_default();
    let detail = format!(
        "virtual_root={}\nconfig={}\n--- generated tsconfig ---\n{generated}\n--- diagnostics ---\n{rendered}\n--- use.ts ---\n{}\n--- widget ---\n{}",
        virtual_root.display(),
        config.display(),
        importer_content,
        widget_content
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == Some(1110)),
        "syntax error must remain\n{detail}"
    );
    assert!(
        result.diagnostics.iter().all(
            |diagnostic| diagnostic.code != Some(2614) && !diagnostic.message.contains("*.vue")
        ),
        "SFC import resolved as the wildcard shim\n{detail}"
    );
    let _ = std::fs::remove_dir_all(&project_root);
}
