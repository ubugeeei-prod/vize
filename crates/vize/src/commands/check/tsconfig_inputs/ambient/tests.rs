use std::path::{Path, PathBuf};

use super::collect_ambient_declaration_files;
use crate::commands::check::tsconfig_inputs::TsconfigInputCache;

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "vize-ambient-{name}-{}-{case_id}",
        std::process::id()
    ))
}

fn relative_paths(root: &Path, files: &[PathBuf]) -> Vec<String> {
    files
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

#[test]
fn ambient_collection_skips_export_only_generated_declaration_modules() {
    let root = unique_case_dir("generated-dts");
    let _ = std::fs::remove_dir_all(&root);
    write(
        &root,
        "types/codegen/schema.d.ts",
        "export enum AimQuestionDisplayKind { Text = 'TEXT' }\nexport type AimQuestion = { kind: AimQuestionDisplayKind };\n",
    );
    write(
        &root,
        "src/globals.d.ts",
        "export {};\ndeclare global { type GlobalTabType = 'a' | 'b'; }\n",
    );
    write(
        &root,
        "src/env.d.ts",
        "declare const APP_VERSION: string;\n",
    );
    write(&root, "src/shims.d.ts", "declare module '*.css';\n");
    write(
        &root,
        "tsconfig.json",
        r#"{
  "include": ["src/**/*.d.ts", "types/codegen/schema.d.ts"]
}"#,
    );

    let project_root = root.canonicalize().unwrap();
    let files = collect_ambient_declaration_files(
        &project_root,
        Some(&project_root.join("tsconfig.json")),
        &mut TsconfigInputCache::default(),
    );

    assert_eq!(
        relative_paths(&project_root, &files),
        vec!["src/env.d.ts", "src/globals.d.ts", "src/shims.d.ts"]
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn ambient_collection_loads_module_declaration_suffixes() {
    let root = unique_case_dir("module-suffixes");
    let _ = std::fs::remove_dir_all(&root);
    write(&root, "src/App.vue", "<template><div /></template>\n");
    write(
        &root,
        "src/env.d.mts",
        "/// <reference path=\"./feature-flags.d.cts\" />\nexport {};\n",
    );
    write(
        &root,
        "src/feature-flags.d.cts",
        "export {};\ndeclare global { interface ImportMeta { vfFeature: boolean } }\n",
    );
    write(
        &root,
        "src/globals.d.mts",
        "declare const APP_VERSION: string;\n",
    );
    write(
        &root,
        "src/generated.d.cts",
        "export type GeneratedOnly = { ok: true };\n",
    );
    write(
        &root,
        "tsconfig.json",
        r#"{
  "include": ["src/**/*"]
}"#,
    );

    let project_root = root.canonicalize().unwrap();
    let files = collect_ambient_declaration_files(
        &project_root,
        Some(&project_root.join("tsconfig.json")),
        &mut TsconfigInputCache::default(),
    );

    assert_eq!(
        relative_paths(&project_root, &files),
        vec!["src/feature-flags.d.cts", "src/globals.d.mts"]
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn ambient_collection_loads_compiler_options_type_packages() {
    let root = unique_case_dir("compiler-options-types");
    let _ = std::fs::remove_dir_all(&root);
    write(&root, "src/App.vue", "<template><div /></template>\n");
    write(
        &root,
        "node_modules/@nuxt/types/package.json",
        r#"{ "types": "app.d.ts" }"#,
    );
    write(
        &root,
        "node_modules/@nuxt/types/app.d.ts",
        "export interface Context { app: unknown }\n",
    );
    write(
        &root,
        "node_modules/nuxt-i18n/package.json",
        r#"{ "types": "index.d.ts" }"#,
    );
    write(
        &root,
        "node_modules/nuxt-i18n/index.d.ts",
        "export {};\ndeclare module \"@nuxt/types\" { interface Context { i18n: unknown } }\n",
    );
    write(
        &root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "types": ["@nuxt/types", "nuxt-i18n"]
  },
  "include": ["src/**/*"]
}"#,
    );

    let project_root = root.canonicalize().unwrap();
    let files = collect_ambient_declaration_files(
        &project_root,
        Some(&project_root.join("tsconfig.json")),
        &mut TsconfigInputCache::default(),
    );

    let relative = relative_paths(&project_root, &files);
    assert!(
        relative.contains(&"node_modules/@nuxt/types/app.d.ts".to_string()),
        "{relative:?}"
    );
    assert!(
        relative.contains(&"node_modules/nuxt-i18n/index.d.ts".to_string()),
        "{relative:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn ambient_collection_loads_compiler_options_module_declaration_packages() {
    let root = unique_case_dir("compiler-options-module-types");
    let _ = std::fs::remove_dir_all(&root);
    write(&root, "src/App.vue", "<template><div /></template>\n");
    write(
        &root,
        "node_modules/modern-env/package.json",
        r#"{ "types": "index.d.mts" }"#,
    );
    write(
        &root,
        "node_modules/modern-env/index.d.mts",
        "/// <reference path=\"./globals.d.cts\" />\nexport {};\n",
    );
    write(
        &root,
        "node_modules/modern-env/globals.d.cts",
        "export {};\ndeclare global { const MODERN_ENV: true }\n",
    );
    write(
        &root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "types": ["modern-env"]
  },
  "include": ["src/**/*"]
}"#,
    );

    let project_root = root.canonicalize().unwrap();
    let files = collect_ambient_declaration_files(
        &project_root,
        Some(&project_root.join("tsconfig.json")),
        &mut TsconfigInputCache::default(),
    );

    let relative = relative_paths(&project_root, &files);
    assert!(
        relative.contains(&"node_modules/modern-env/index.d.mts".to_string()),
        "{relative:?}"
    );
    assert!(
        relative.contains(&"node_modules/modern-env/globals.d.cts".to_string()),
        "{relative:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn ambient_collection_resolves_compiler_options_types_from_tsconfig_dir() {
    let root = unique_case_dir("compiler-options-types-nested");
    let _ = std::fs::remove_dir_all(&root);
    let app_dir = root.join("packages/app");
    write(&app_dir, "src/App.vue", "<template><div /></template>\n");
    write(
        &app_dir,
        "node_modules/vue/package.json",
        r#"{ "exports": { "./jsx": "./jsx.d.ts" } }"#,
    );
    write(
        &app_dir,
        "node_modules/vue/jsx.d.ts",
        "export {};\ndeclare global { namespace JSX { interface IntrinsicElements { div: unknown } } }\n",
    );
    write(
        &app_dir,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "types": ["vue/jsx"]
  },
  "include": ["src/**/*"]
}"#,
    );

    let project_root = root.canonicalize().unwrap();
    let files = collect_ambient_declaration_files(
        &project_root,
        Some(&project_root.join("packages/app/tsconfig.json")),
        &mut TsconfigInputCache::default(),
    );

    assert!(
        relative_paths(&project_root, &files)
            .contains(&"packages/app/node_modules/vue/jsx.d.ts".to_string()),
        "{files:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
