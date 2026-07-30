//! Registry dependencies installed only in a sub-package's `node_modules`
//! (#3366).
//!
//! A pnpm workspace links a package's own dependencies into
//! `<package>/node_modules`, not the workspace root's. When the effective
//! tsconfig lives at the workspace root, the virtual project anchors there
//! and a mirrored file's ancestor walk never passes the sub-package's real
//! `node_modules`, so a bare import died with a spurious TS2307 even though
//! `tsgo` on the same tsconfig resolves it. The mirror now symlinks each
//! per-package `node_modules` into the matching project-relative location,
//! so the dependency's type contract is enforced instead of excused.

use std::path::{Path, PathBuf};

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};

const MAIN: &str = r#"import { answer } from 'acme';
export const bad: string = answer;
"#;

#[test]
fn subpackage_registry_dependency_enforces_its_contract() {
    let project = create_workspace();
    let diagnostics = project_diagnostics(project.path());

    assert!(
        diagnostics.contains(&Some(2322)),
        "the resolved dependency must enforce its contract (string ≠ number): {diagnostics:?}"
    );
    assert!(
        !diagnostics.contains(&Some(2307)),
        "the dependency exists in the sub-package's node_modules and must resolve: {diagnostics:?}"
    );

    let mirror = project
        .path()
        .join("node_modules/.vize/canon/apps/app/node_modules");
    assert!(
        std::fs::symlink_metadata(&mirror)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false),
        "the sub-package node_modules must be mirrored as a symlink at {}",
        mirror.display()
    );
}

fn project_diagnostics(root: &Path) -> Vec<Option<u32>> {
    let mut checker = BatchTypeChecker::new(root).expect("type checker should start");
    checker.scan_project().expect("project should scan");
    checker
        .check_project()
        .expect("project should type check")
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

fn create_workspace() -> tempfile::TempDir {
    let project = tempfile::tempdir().expect("temporary project should be created");
    write_file(
        project.path(),
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "types": []
  },
  "include": ["apps/**/*"]
}"#,
    );
    // Ambient vue runtime stub, same as the other canon integration fixtures.
    write_file(
        project.path(),
        "node_modules/vue/package.json",
        r#"{ "name": "vue", "types": "index.d.ts" }"#,
    );
    write_file(
        project.path(),
        "node_modules/vue/index.d.ts",
        "export interface ComponentPublicInstance {}\n",
    );
    // The registry dependency exists ONLY in the sub-package's node_modules.
    write_file(
        project.path(),
        "apps/app/node_modules/acme/package.json",
        r#"{ "name": "acme", "types": "index.d.ts" }"#,
    );
    write_file(
        project.path(),
        "apps/app/node_modules/acme/index.d.ts",
        "export declare const answer: number;\n",
    );
    write_file(project.path(), "apps/app/src/main.ts", MAIN);
    project
}

fn write_file(root: &Path, path: &str, source: &str) {
    let path: PathBuf = root.join(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent directory should be created");
    }
    std::fs::write(path, source).expect("fixture should be written");
}
