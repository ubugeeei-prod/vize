//! Solution shells defer their import closure to referenced programs.

use std::path::{Path, PathBuf};

use vize_carton::path::canonicalize_non_verbatim;

fn write(root: &Path, rel: &str, content: &str) -> PathBuf {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, content).unwrap();
    canonicalize_non_verbatim(&path)
}

fn collect(root: &Path, cwd: &Path, tsconfig: &Path) -> super::CollectedRoots {
    super::collect_default_roots(
        super::DefaultRunFileContext {
            project_root: root,
            cwd,
            tsconfig_path: Some(tsconfig),
            import_options: super::ImportFileOptions::default(),
            check_ignore_set: None,
        },
        &mut super::TsconfigInputCache::default(),
        &mut super::CanonicalPathCache::default(),
        &mut vize_canon::PackageRouteResolver::default(),
    )
}

#[test]
fn splitting_shell_leaves_import_closures_to_its_programs() {
    let dir = tempfile::tempdir().unwrap();
    let root = canonicalize_non_verbatim(dir.path());
    let shell = write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./tsconfig.app.json"},{"path":"./tsconfig.server.json"}]}"#,
    );
    write(&root, "tsconfig.app.json", r#"{"include":["app/**/*.ts"]}"#);
    write(
        &root,
        "tsconfig.server.json",
        r#"{"include":["server/**/*.ts"]}"#,
    );
    let app = write(
        &root,
        "app/use.ts",
        "export { value } from '../lib/value'\n",
    );
    let server = write(&root, "server/api.ts", "export const api = 1\n");
    let lib = write(&root, "lib/value.ts", "export const value = 1\n");

    // Each referenced program walks its own imports; the shell's walk was dropped.
    let split = collect(&root, &root, &shell);
    assert_eq!(split.files, vec![app.clone(), server.clone()]);
    assert_eq!(split.inputs, split.files);
    assert!(split.package_routes.is_empty());

    // A scope report below the project root still counts the shell's closure.
    let nested = collect(&root, &root.join("app"), &shell);
    assert!(nested.files.contains(&lib));

    // One program keeps the closure it is checked with.
    let single = collect(&root, &root, &root.join("tsconfig.app.json"));
    assert!(single.files.contains(&app) && single.files.contains(&lib));
}
