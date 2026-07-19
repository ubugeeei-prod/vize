use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait, SfcTypeCheckOptions, type_check_sfc};

const COMPONENT: &str = r#"<script setup lang="ts">
import type { Payload } from "virtual-service";

declare global {
  interface Window {
    refreshLivePreview: (url: string | null) => void;
  }
}

declare module "virtual-service" {
  interface Payload {
    label: string;
  }
}

const payload: Payload = { id: "one", label: "ready" };
window.refreshLivePreview("/preview");
void payload;
</script>

<template><div /></template>
"#;

#[test]
fn ambient_declarations_are_emitted_once_at_module_scope() {
    let virtual_ts = type_check_sfc(
        COMPONENT,
        &SfcTypeCheckOptions::new("LivePreview.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated");

    let setup_start = virtual_ts
        .find("function __setup")
        .expect("setup function should be generated");
    let global_declaration = "declare global {\n  interface Window";
    let global_start = virtual_ts
        .find(global_declaration)
        .expect("global augmentation should be preserved");
    let module_start = virtual_ts
        .find("declare module \"virtual-service\"")
        .expect("module augmentation should be preserved");
    assert!(
        global_start < setup_start && module_start < setup_start,
        "ambient declarations must precede the setup function:\n{virtual_ts}"
    );
    assert_eq!(virtual_ts.match_indices(global_declaration).count(), 1);
    assert_eq!(
        virtual_ts
            .match_indices("declare module \"virtual-service\"")
            .count(),
        1
    );
    assert!(
        virtual_ts.contains("refreshLivePreview: (url: string | null) => void;"),
        "the global declaration body must remain byte-complete:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("interface Payload {\n    label: string;\n  }"),
        "the module augmentation body must remain byte-complete:\n{virtual_ts}"
    );

    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, virtual_ts.as_str(), oxc_span::SourceType::ts())
            .parse();
    assert!(
        !parsed.panicked && parsed.errors.is_empty(),
        "hoisted ambient declarations must produce parseable TypeScript: {:#?}\n{virtual_ts}",
        parsed.errors
    );
}

#[test]
fn ambient_augmentations_accept_valid_uses_and_reject_invalid_ones() {
    let project = tempfile::tempdir().expect("temporary project should be created");
    write_project(project.path());

    let mut checker = BatchTypeChecker::new(project.path()).expect("type checker should start");
    checker.scan_project().expect("project should scan");
    let diagnostics = checker
        .check_project()
        .expect("project should type check")
        .diagnostics;

    let structural_failures: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(diagnostic.code, Some(1234 | 2307 | 2339 | 2664))
                || (diagnostic.file.ends_with("consumer.ts")
                    && !matches!(diagnostic.code, Some(2345 | 2741)))
        })
        .collect();
    assert!(
        structural_failures.is_empty(),
        "valid global and module augmentations must remain diagnostic-free: {structural_failures:#?}"
    );

    let rejected_uses: Vec<_> = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.file.ends_with("consumer.ts") && matches!(diagnostic.code, Some(2345 | 2741))
        })
        .collect();
    assert_eq!(
        rejected_uses.len(),
        2,
        "both invalid uses must be rejected without widening the augmentations: {diagnostics:#?}"
    );
}

fn write_project(root: &Path) {
    write_file(
        root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    );
    write_file(
        root,
        "node_modules/vue/package.json",
        r#"{ "name": "vue", "types": "index.d.ts" }"#,
    );
    write_file(
        root,
        "node_modules/vue/index.d.ts",
        r#"export interface ComponentPublicInstance {
  $attrs: Record<string, unknown>;
  $slots: Record<string, unknown>;
  $refs: Record<string, unknown>;
  $emit: (...args: unknown[]) => void;
}
"#,
    );
    write_file(
        root,
        "node_modules/virtual-service/package.json",
        r#"{ "name": "virtual-service", "types": "index.d.ts" }"#,
    );
    write_file(
        root,
        "node_modules/virtual-service/index.d.ts",
        "export interface Payload { id: string; }\n",
    );
    write_file(root, "src/LivePreview.vue", COMPONENT);
    write_file(
        root,
        "src/consumer.ts",
        r#"import type { Payload } from "virtual-service";

const accepted: Payload = { id: "one", label: "ready" };
const rejected: Payload = { id: "two" };
window.refreshLivePreview("/ready");
window.refreshLivePreview(42);
void accepted;
void rejected;
"#,
    );
}

fn write_file(root: &Path, path: &str, source: &str) {
    let path = root.join(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent directory should be created");
    }
    std::fs::write(path, source).expect("fixture should be written");
}
