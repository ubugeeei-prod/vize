//! Nuxt manifest filtering tests split out from the large tsconfig suite.

#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use vize_s0::cstr;

use super::TsconfigInputCache;

fn collect_default_check_files(project_root: &Path, tsconfig_path: &Path) -> Vec<PathBuf> {
    super::collect_default_check_files(
        project_root,
        Some(tsconfig_path),
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
fn default_collection_skips_nuxt_module_import_manifest_files_entries() {
    let case_dir = unique_case_dir("tsconfig-nuxt-module-import-manifest-files");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join(".nuxt/types")).unwrap();
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/App.vue"), "<template />").unwrap();
    fs::write(
        case_dir.join(".nuxt/imports.d.mts"),
        "export { useModuleImport } from '../composables/useModuleImport'\n",
    )
    .unwrap();
    fs::write(
        case_dir.join(".nuxt/types/imports.d.cts"),
        "declare global { const useModuleImport: () => string }\nexport {}\n",
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "files": [
    "src/App.vue",
    ".nuxt/imports.d.mts",
    ".nuxt/types/imports.d.cts"
  ]
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, &case_dir.join("tsconfig.json"));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/App.vue"]);

    let _ = fs::remove_dir_all(&case_dir);
}
