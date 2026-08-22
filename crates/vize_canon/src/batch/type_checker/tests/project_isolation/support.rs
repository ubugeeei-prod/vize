use super::BatchTypeChecker;
use crate::batch::{TypeCheckResult, TypeChecker};
use std::{
    path::Path,
    sync::{Arc, Barrier},
    thread,
};

pub(super) fn create_project(project_root: &Path, owner: &str) {
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "baseUrl": ".",
    "paths": { "@owner": ["./src/owner.ts"] }
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/owner.ts"),
        format!("export const owner = '{owner}' as const\n"),
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/public.ts"),
        format!("export const marker = '{owner}' as const\n"),
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/App.vue"),
        format!(
            "<script setup lang=\"ts\">\nimport {{ owner }} from '@owner'\nconst checked: '{owner}' = owner\n</script>\n\n<template><p>{{{{ checked }}}}</p></template>\n"
        ),
    )
    .unwrap();
}

pub(super) fn create_relative_project(project_root: &Path, owner: &str) {
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
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
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/owner.ts"),
        format!("export type Owner = '{owner}'\n"),
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/App.vue"),
        format!(
            "<script setup lang=\"ts\">\nimport type {{ Owner }} from './owner'\nconst checked: Owner = '{owner}'\n</script>\n\n<template><p>{{{{ checked }}}}</p></template>\n"
        ),
    )
    .unwrap();
}

pub(super) fn populate_shared_dependency_tree(shared_node_modules: &Path) {
    let workspace_node_modules = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .join("node_modules");
    std::fs::create_dir_all(shared_node_modules).unwrap();
    for entry in std::fs::read_dir(workspace_node_modules).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == ".vize" {
            continue;
        }
        std::os::unix::fs::symlink(entry.path(), shared_node_modules.join(entry.file_name()))
            .unwrap();
    }
}

pub(super) fn link_shared_node_modules(project_root: &Path, shared_node_modules: &Path) {
    std::os::unix::fs::symlink(shared_node_modules, project_root.join("node_modules")).unwrap();
}

pub(super) fn scanned_checker(project_root: &Path) -> BatchTypeChecker {
    let mut checker = BatchTypeChecker::new(project_root).expect("checker should start");
    checker.scan_project().expect("project should scan");
    checker
}

pub(super) fn assert_canonical_virtual_paths(
    checker: &BatchTypeChecker,
    project_root: &Path,
    shared_node_modules: &Path,
) {
    let virtual_root = crate::batch::project_virtual_root(project_root);
    let canonical_storage = std::fs::canonicalize(shared_node_modules).unwrap();
    assert!(
        !virtual_root.starts_with(&canonical_storage),
        "mutable project storage must not live in the shared dependency tree"
    );
    assert!(
        virtual_root
            .components()
            .all(|component| component.as_os_str() != "node_modules"),
        "mutable project storage must stay outside node_modules"
    );
    assert!(
        checker
            .virtual_files()
            .iter()
            .all(|file| file.virtual_path.starts_with(&virtual_root)),
        "Corsa workspace and document paths must use the same canonical storage spelling"
    );
}

pub(super) fn concurrent_checks(
    first: &BatchTypeChecker,
    second: &BatchTypeChecker,
) -> (TypeCheckResult, TypeCheckResult) {
    thread::scope(|scope| {
        let barrier = Arc::new(Barrier::new(2));
        let first_barrier = Arc::clone(&barrier);
        let first_task = scope.spawn(move || {
            first_barrier.wait();
            first.check_project()
        });
        let second_task = scope.spawn(move || {
            barrier.wait();
            second.check_project()
        });
        (
            first_task
                .join()
                .unwrap()
                .expect("first check should complete"),
            second_task
                .join()
                .unwrap()
                .expect("second check should complete"),
        )
    })
}

pub(super) fn assert_clean_and_owned(
    result: &TypeCheckResult,
    owner_root: &Path,
    other_root: &Path,
    phase: &str,
) {
    assert!(
        result.success,
        "{phase} should be clean: {:#?}",
        result.diagnostics
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.file.starts_with(other_root)),
        "{phase} leaked diagnostics from another project: {:#?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics.iter().all(|diagnostic| {
            diagnostic.file.starts_with(owner_root) || diagnostic.file == owner_root
        }),
        "{phase} reported an unowned path: {:#?}",
        result.diagnostics
    );
}

pub(super) fn assert_materialized_source(checker: &BatchTypeChecker, expected: &str) {
    let app = checker
        .virtual_files()
        .into_iter()
        .find(|file| file.original_path.ends_with("src/App.vue"))
        .expect("App.vue should be registered");
    let materialized = std::fs::read_to_string(&app.virtual_path).unwrap();
    assert!(
        materialized.contains(expected),
        "materialized project state was overwritten: expected {expected:?} in {}",
        app.virtual_path.display()
    );
}

pub(super) fn assert_type_error(result: &TypeCheckResult, app: &Path) {
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.file == app && diagnostic.code == Some(2322))
        .unwrap_or_else(|| {
            panic!(
                "expected authored TS2322 for {}: {result:#?}",
                app.display()
            )
        });
    assert_eq!((diagnostic.line, diagnostic.column), (2, 6));
}

pub(super) fn assert_declaration_owner(
    declarations: Vec<crate::batch::DeclarationOutput>,
    expected: &str,
    forbidden: &str,
) {
    let public = declarations
        .iter()
        .find(|declaration| declaration.path.ends_with("public.d.ts"))
        .unwrap_or_else(|| panic!("public.d.ts was not emitted: {declarations:#?}"));
    assert!(public.content.contains(expected), "{public:#?}");
    assert!(!public.content.contains(forbidden), "{public:#?}");
}
