//! Generated GraphQL declaration collection regressions.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::fs;
use std::path::{Path, PathBuf};

use super::TsconfigInputCache;
use vize_carton::cstr;

fn collect_default_check_files(project_root: &Path, tsconfig_path: Option<&Path>) -> Vec<PathBuf> {
    super::collect_default_check_files(
        project_root,
        tsconfig_path,
        false,
        &mut TsconfigInputCache::default(),
    )
}

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
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
fn default_collection_skips_generated_codegen_declaration_modules() {
    let case_dir = unique_case_dir("tsconfig-generated-codegen-dts");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::create_dir_all(case_dir.join("types/codegen")).unwrap();
    fs::write(case_dir.join("src/env.d.ts"), "declare const X: string;").unwrap();
    fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
    fs::write(
        case_dir.join("types/codegen/schema.d.ts"),
        "export enum AimQuestionDisplayKind { Text = 'TEXT' }\n",
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["src/**/*.vue", "src/**/*.d.ts", "types/codegen/schema.d.ts"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec!["src/App.vue", "src/env.d.ts"]
    );

    let _ = fs::remove_dir_all(&case_dir);
}
