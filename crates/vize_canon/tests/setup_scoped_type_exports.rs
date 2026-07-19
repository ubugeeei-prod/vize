use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait, SfcTypeCheckOptions, type_check_sfc};

const COMPONENT: &str = r#"<script lang="ts">
const NATIVE_TYPES = ["button", "submit", "reset"] as const;
export type ButtonNativeType = typeof NATIVE_TYPES[number];

const COLORS = ["red", "blue"] as const;
export interface BadgeOptions {
  color: typeof COLORS[number];
}

const selected: ButtonNativeType = "button";
const options: BadgeOptions = { color: "red" };
void selected;
void options;
</script>

<template><button /></template>
"#;

const SETUP_PROPS_COMPONENT: &str = r#"<script setup lang="ts">
const NATIVE_TYPES = ["button", "submit"] as const;
export interface Props {
  nativeType: typeof NATIVE_TYPES[number];
}
const props = defineProps<Props>();
void props;
</script>

<template><button /></template>
"#;

const PRIVATE_SETUP_PROPS_COMPONENT: &str = r#"<script setup lang="ts">
const NATIVE_TYPES = ["button", "submit"] as const;
type Props = {
  nativeType: typeof NATIVE_TYPES[number];
};
const props = defineProps<Props>();
void props;
</script>

<template><button /></template>
"#;

#[test]
fn setup_value_dependent_exports_are_captured_and_reexported() {
    let virtual_ts = type_check_sfc(
        COMPONENT,
        &SfcTypeCheckOptions::new("Button.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated");

    assert!(
        !virtual_ts.contains("  export type ButtonNativeType"),
        "an export modifier must not remain inside __setup:\n{virtual_ts}"
    );
    assert!(
        !virtual_ts.contains("  export interface BadgeOptions"),
        "an interface export modifier must not remain inside __setup:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("type ButtonNativeType = typeof NATIVE_TYPES[number]"),
        "the setup-scoped alias must retain its value dependency:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("color: typeof COLORS[number]"),
        "the setup-scoped interface body must remain byte-complete:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains(
            "export type ButtonNativeType = Awaited<ReturnType<typeof __setup>>[\"__vize_exported_type_ButtonNativeType\"]"
        ),
        "the public alias must be restored at module scope:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains(
            "export type BadgeOptions = Awaited<ReturnType<typeof __setup>>[\"__vize_exported_type_BadgeOptions\"]"
        ),
        "the public interface shape must be restored at module scope:\n{virtual_ts}"
    );

    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, virtual_ts.as_str(), oxc_span::SourceType::ts())
            .parse();
    assert!(
        !parsed.panicked && parsed.errors.is_empty(),
        "captured type exports must produce parseable TypeScript: {:#?}\n{virtual_ts}",
        parsed.errors
    );
}

#[test]
fn downstream_imports_accept_exact_types_and_reject_invalid_values() {
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
            matches!(diagnostic.code, Some(1184 | 2305 | 2300 | 2614))
                || (diagnostic.file.ends_with("consumer.ts") && diagnostic.code != Some(2322))
        })
        .collect();
    assert!(
        structural_failures.is_empty(),
        "valid public type imports must remain diagnostic-free: {structural_failures:#?}"
    );

    let rejected_assignments = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.file.ends_with("consumer.ts") && diagnostic.code == Some(2322)
        })
        .count();
    assert_eq!(
        rejected_assignments, 2,
        "both invalid literal assignments must be rejected without widening: {diagnostics:#?}"
    );
}

#[test]
fn exported_setup_scoped_props_have_one_public_declaration() {
    let virtual_ts = type_check_sfc(
        SETUP_PROPS_COMPONENT,
        &SfcTypeCheckOptions::new("SetupButton.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated");

    assert_eq!(
        virtual_ts.match_indices("export type Props =").count(),
        1,
        "the setup artifact and the public source type must not emit duplicate Props aliases:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains("type __VizeResolvedProps = Awaited<ReturnType<typeof __setup>>"),
        "component prop checking must still resolve the setup-scoped Props shape:\n{virtual_ts}"
    );

    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, virtual_ts.as_str(), oxc_span::SourceType::ts())
            .parse();
    assert!(
        parsed.errors.is_empty(),
        "a public setup-scoped Props interface must remain parseable: {:#?}\n{virtual_ts}",
        parsed.errors
    );
}

#[test]
fn private_setup_scoped_props_still_export_a_public_declaration() {
    let virtual_ts = type_check_sfc(
        PRIVATE_SETUP_PROPS_COMPONENT,
        &SfcTypeCheckOptions::new("PrivateSetupButton.vue").with_virtual_ts(),
    )
    .virtual_ts
    .expect("virtual TypeScript should be generated");

    // The user's `Props` is private and value-dependent, so it stays inside
    // `__setup` and is never re-exported by the type-export plan. The public
    // `export type Props` consumers need must still be restored from the setup
    // return, and there must be exactly one of them.
    assert_eq!(
        virtual_ts.match_indices("export type Props =").count(),
        1,
        "a private value-dependent Props must still yield one public export:\n{virtual_ts}"
    );
    assert!(
        virtual_ts.contains(
            "export type Props = Awaited<ReturnType<typeof __setup>>[\"__vize_setup_props\"]"
        ),
        "the public Props alias must resolve from the setup return:\n{virtual_ts}"
    );
    assert!(
        !virtual_ts.contains("__VizeResolvedProps"),
        "a private Props must not be treated as an existing public alias:\n{virtual_ts}"
    );

    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, virtual_ts.as_str(), oxc_span::SourceType::ts())
            .parse();
    assert!(
        parsed.errors.is_empty(),
        "a private setup-scoped Props must remain parseable: {:#?}\n{virtual_ts}",
        parsed.errors
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
    write_file(root, "src/Button.vue", COMPONENT);
    write_file(
        root,
        "src/consumer.ts",
        r#"import type { BadgeOptions, ButtonNativeType } from "./Button.vue";

const acceptedButton: ButtonNativeType = "submit";
const rejectedButton: ButtonNativeType = "anchor";
const acceptedBadge: BadgeOptions = { color: "blue" };
const rejectedBadge: BadgeOptions = { color: "green" };

void acceptedButton;
void rejectedButton;
void acceptedBadge;
void rejectedBadge;
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
