#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use std::path::{Path, PathBuf};

use vize_s0::path::canonicalize_non_verbatim;

/// Per-test scratch directory under `target/vize-tests`, unique per process and case.
fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join(format!(
            "check-runner-{name}-{}-{case_id}",
            std::process::id()
        ))
}

/// Write `content` at `root/rel`, creating parent directories, and return the path.
fn write(root: &Path, rel: &str, content: &str) -> PathBuf {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, content).unwrap();
    path
}

/// Explicit runs (`vize check src/a.ts`) collect ambient declarations through
/// `register_explicit_ambient_imports`. A tsconfig `files` entry that lives
/// outside the nearest package root is still a program member, exactly as it
/// is for default runs (#5629).
#[test]
fn explicit_run_keeps_tsconfig_files_entry_outside_nearest_package_root() {
    let workspace = unique_case_dir("explicit-ambient-outside-package");
    let _ = std::fs::remove_dir_all(&workspace);
    write(&workspace, "package.json", r#"{ "private": true }"#);
    write(&workspace, "app/package.json", r#"{ "private": true }"#);
    write(
        &workspace,
        "app/tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "files": ["../shared/globals.d.ts"],
  "include": ["src/**/*"]
}"#,
    );
    let shared = write(
        &workspace,
        "shared/globals.d.ts",
        r#"interface AmbientPayload {
  label: string;
}

declare const ambientPayload: AmbientPayload;
"#,
    );
    let entry = write(
        &workspace,
        "app/src/a.ts",
        "const label: string = ambientPayload.label;\nexport {};\n",
    );

    let project_root = workspace.join("app").canonicalize().unwrap();
    let mut files = vec![canonicalize_non_verbatim(&entry)];
    let mut tsconfig_input_cache = super::super::TsconfigInputCache::default();
    let mut canonical_paths = super::super::CanonicalPathCache::default();
    let mut package_routes = vize_canon::PackageRouteResolver::default();

    super::super::register_explicit_ambient_imports(
        &mut files,
        super::super::ExplicitAmbientImportContext::new(
            &project_root,
            &project_root,
            &project_root.join("tsconfig.json"),
            &project_root,
            &[],
            super::super::ImportFileOptions::default(),
        ),
        &mut tsconfig_input_cache,
        &mut canonical_paths,
        &mut package_routes,
    );

    let shared = canonicalize_non_verbatim(&shared);
    assert!(
        files.contains(&shared),
        "tsconfig `files` entry outside the package root must stay a program member: {files:?}"
    );

    let _ = std::fs::remove_dir_all(&workspace);
}
