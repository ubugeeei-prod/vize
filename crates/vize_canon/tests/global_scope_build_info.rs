//! Corsa 7.0.2 keeps unchanged files' recorded diagnostics when a global
//! declaration joins the program or changes (#10819). Consecutive checks of
//! one project share its build info, so the second check must not reuse them.
#![expect(clippy::expect_used, reason = "tests assert by panicking")]

use std::path::Path;

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait};
use vize_carton::String;

const GLOBAL_USE: &str = r#"<script setup lang="ts">
const value: ScopedGlobal = "x";
void value;
</script>

<template><div /></template>
"#;

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("a file has a parent"))
        .expect("parents must be created");
    std::fs::write(path, content).expect("the file must be written");
}

fn project(root: &Path) {
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true,"skipLibCheck":true,"types":[]},"include":["src"]}"#,
    );
    let vue = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/node_modules/vue");
    let vue = std::fs::canonicalize(vue).expect("the pinned real Vue package must be installed");
    std::fs::create_dir_all(root.join("node_modules")).expect("node_modules must be created");
    #[cfg(unix)]
    std::os::unix::fs::symlink(vue, root.join("node_modules/vue")).expect("Vue must link");
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(vue, root.join("node_modules/vue")).expect("Vue must link");
    write(root, "src/Page.vue", GLOBAL_USE);
}

fn check(root: &Path) -> Vec<(String, Option<u32>, String)> {
    let mut checker = BatchTypeChecker::new(root).expect("the checker must start");
    checker.scan_project().expect("the project must scan");
    checker
        .check_project()
        .expect("the project must check")
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            let file = diagnostic
                .file
                .file_name()
                .expect("a diagnostic has a file");
            (
                file.to_string_lossy().into(),
                diagnostic.code,
                diagnostic.message,
            )
        })
        .collect()
}

#[test]
fn newly_imported_global_declarations_recheck_unchanged_files() {
    let root = tempfile::tempdir().expect("a project must be created");
    let root = root.path();
    project(root);
    write(
        root,
        "node_modules/scoped-globals/package.json",
        r#"{"name":"scoped-globals","types":"index.d.ts"}"#,
    );
    write(
        root,
        "node_modules/scoped-globals/index.d.ts",
        "export {};\ndeclare global { type ScopedGlobal = string; }\n",
    );
    write(root, "src/entry.ts", "export const entry = 1;\n");
    assert_eq!(
        check(root),
        [(
            "Page.vue".into(),
            Some(2304),
            "Cannot find name 'ScopedGlobal'.".into()
        )]
    );
    write(
        root,
        "src/entry.ts",
        "import \"scoped-globals\";\nexport const entry = 1;\n",
    );
    assert_eq!(check(root), []);
}

#[test]
fn edited_global_declarations_recheck_unchanged_files() {
    let root = tempfile::tempdir().expect("a project must be created");
    let root = root.path();
    project(root);
    write(root, "src/globals.d.ts", "type ScopedGlobal = string;\n");
    assert_eq!(check(root), []);
    write(root, "src/globals.d.ts", "type ScopedGlobal = number;\n");
    assert_eq!(
        check(root),
        [(
            "Page.vue".into(),
            Some(2322),
            "Type 'string' is not assignable to type 'number'.".into()
        )]
    );
}
