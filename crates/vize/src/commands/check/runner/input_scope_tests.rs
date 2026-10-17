//! Scope classification for default `vize check` runs (#3320).
//!
//! Every case builds a real layout on disk — links included — because the
//! classification compares resolved paths, which only a real filesystem
//! answers. Workspaces live under `std::env::temp_dir()` so the tests run from
//! a clean checkout with no build artifacts.
//! Authored configuration dependencies are validated before those scope filters.

use std::{
    fs,
    path::{Path, PathBuf},
};
use vize_l0::cstr;

use super::{
    DefaultRunScope, classify_default_run_scope, unowned_project_error, validate_tsconfig_chain,
    widened_scope_note,
};

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir()
        .join(vize_l0::cstr!("vize-input-scope-{name}-{}-{case_id}", std::process::id()).as_str())
}

fn symlink_dir(source: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}

/// Ancestor `tsconfig.json`, nested app with no config of its own, and the
/// ancestor's program covering only its own sources: the #3320 layout. The app
/// directory is not part of the resolved project, so a run there must not be
/// reported as that project's result.
#[test]
fn nested_directory_outside_the_resolved_program_is_unowned() {
    let workspace = unique_case_dir("unowned");
    let app = workspace.join("nested/app");
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::create_dir_all(workspace.join("helpers")).unwrap();
    std::fs::write(workspace.join("tsconfig.json"), "{}").unwrap();
    let helpers = ["h1.ts", "h2.ts", "h3.ts"].map(|name| {
        let path = workspace.join("helpers").join(name);
        std::fs::write(&path, "export const helper = 1;\n").unwrap();
        path
    });
    std::fs::write(app.join("src/Broken.vue"), "<template />").unwrap();

    assert_eq!(
        classify_default_run_scope(&helpers, &app, &workspace),
        DefaultRunScope::Unowned { inputs: 3 }
    );

    let _ = std::fs::remove_dir_all(&workspace);
}

/// A `package.json` in the nested directory does not change the verdict: a
/// package boundary declares a package, not a type-checking program, and the
/// resolved project still contains none of its files.
#[test]
fn package_json_in_the_nested_directory_is_still_unowned() {
    let workspace = unique_case_dir("unowned-package");
    let app = workspace.join("nested/app");
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::write(workspace.join("tsconfig.json"), "{}").unwrap();
    std::fs::write(app.join("package.json"), "{}").unwrap();
    let inputs = vec![workspace.join("helper.ts")];
    std::fs::write(&inputs[0], "export const helper = 1;\n").unwrap();

    assert_eq!(
        classify_default_run_scope(&inputs, &app, &workspace),
        DefaultRunScope::Unowned { inputs: 1 }
    );

    let _ = std::fs::remove_dir_all(&workspace);
}

/// `node_modules` symlinked to a sibling directory inside the project. The
/// dependency input resolves to the sibling spelling, outside the working
/// directory, while the working directory's own source keeps the run owned.
#[test]
fn node_modules_symlinked_to_a_sibling_keeps_the_run_owned() {
    let workspace = unique_case_dir("sibling-store");
    let store = workspace.join("vendor");
    std::fs::create_dir_all(store.join("acme")).unwrap();
    std::fs::create_dir_all(workspace.join("src")).unwrap();
    symlink_dir(&store, &workspace.join("node_modules")).unwrap();
    let source = workspace.join("src/App.vue");
    let dependency = workspace.join("node_modules/acme/index.ts");
    std::fs::write(&source, "<template />").unwrap();
    std::fs::write(store.join("acme/index.ts"), "export const acme = 1;\n").unwrap();

    assert_eq!(
        classify_default_run_scope(&[source, dependency], &workspace.join("src"), &workspace),
        DefaultRunScope::Widened {
            inputs: 2,
            outside: 1
        }
    );

    let _ = std::fs::remove_dir_all(&workspace);
}

/// `node_modules` symlinked to a store outside the project — a pnpm store, a
/// hoisting shim, a bind-mounted dependency tree. The dependency's resolved
/// path leaves the project entirely and must not make the run unowned when the
/// working directory's own sources are in the program.
#[test]
fn node_modules_symlinked_outside_the_project_keeps_the_run_owned() {
    let workspace = unique_case_dir("outside-store");
    let store = unique_case_dir("outside-store-target");
    std::fs::create_dir_all(store.join("acme")).unwrap();
    std::fs::create_dir_all(workspace.join("app/src")).unwrap();
    symlink_dir(&store, &workspace.join("node_modules")).unwrap();
    std::fs::write(workspace.join("tsconfig.json"), "{}").unwrap();
    let source = workspace.join("app/src/App.vue");
    let dependency = workspace.join("node_modules/acme/index.ts");
    std::fs::write(&source, "<template />").unwrap();
    std::fs::write(store.join("acme/index.ts"), "export const acme = 1;\n").unwrap();

    assert_eq!(
        classify_default_run_scope(&[source, dependency], &workspace.join("app"), &workspace),
        DefaultRunScope::Widened {
            inputs: 2,
            outside: 1
        }
    );

    let _ = std::fs::remove_dir_all(&workspace);
    let _ = std::fs::remove_dir_all(&store);
}

/// pnpm's isolated linker shape: `node_modules` is a real directory and the
/// individual package inside it is the link. The package's resolved path is
/// outside the working directory; the working directory's own sources are not.
#[test]
fn symlinked_package_inside_node_modules_keeps_the_run_owned() {
    let workspace = unique_case_dir("pnpm-package");
    let store = workspace.join(".pnpm/acme@1.0.0/node_modules/acme");
    std::fs::create_dir_all(&store).unwrap();
    std::fs::create_dir_all(workspace.join("app/src")).unwrap();
    std::fs::create_dir_all(workspace.join("node_modules")).unwrap();
    symlink_dir(&store, &workspace.join("node_modules/acme")).unwrap();
    let source = workspace.join("app/src/App.vue");
    let dependency = workspace.join("node_modules/acme/index.ts");
    std::fs::write(&source, "<template />").unwrap();
    std::fs::write(store.join("index.ts"), "export const acme = 1;\n").unwrap();

    assert_eq!(
        classify_default_run_scope(&[source, dependency], &workspace.join("app"), &workspace),
        DefaultRunScope::Widened {
            inputs: 2,
            outside: 1
        }
    );

    let _ = std::fs::remove_dir_all(&workspace);
}

/// The whole project reached through a symlinked path: the working directory is
/// spelled through the link while the collected inputs carry the resolved
/// spelling. A spelling comparison would call every input "outside" and fail
/// the run; the resolved comparison keeps it owned.
#[test]
fn project_reached_through_a_symlinked_path_is_owned() {
    let workspace = unique_case_dir("linked-project");
    let link = unique_case_dir("linked-project-link");
    std::fs::create_dir_all(workspace.join("src")).unwrap();
    std::fs::write(workspace.join("tsconfig.json"), "{}").unwrap();
    let source = workspace.join("src/App.vue");
    std::fs::write(&source, "<template />").unwrap();
    symlink_dir(&workspace, &link).unwrap();

    assert_eq!(
        classify_default_run_scope(&[source], &link.join("src"), &link),
        DefaultRunScope::Owned
    );

    let _ = std::fs::remove_file(&link);
    let _ = std::fs::remove_dir_all(&workspace);
}

/// A `tsconfig.json` in both the working directory and an ancestor: discovery
/// stops at the nearest one, so the root is the working directory and there is
/// nothing to surface.
#[test]
fn nearest_tsconfig_root_is_owned() {
    let workspace = unique_case_dir("nearest");
    let app = workspace.join("app");
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::write(workspace.join("tsconfig.json"), "{}").unwrap();
    std::fs::write(app.join("tsconfig.json"), "{}").unwrap();
    let source = app.join("src/App.vue");
    std::fs::write(&source, "<template />").unwrap();

    assert_eq!(
        classify_default_run_scope(&[source], &app, &app),
        DefaultRunScope::Owned
    );

    let _ = std::fs::remove_dir_all(&workspace);
}

/// The negative direction: no config in the working directory and an ancestor
/// program that does own its sources. Walking up is correct here, and the run
/// must proceed — only the wider scope is surfaced.
#[test]
fn ancestor_program_that_owns_the_working_directory_is_widened_not_unowned() {
    let workspace = unique_case_dir("widened");
    let app = workspace.join("nested/app");
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::create_dir_all(workspace.join("helpers")).unwrap();
    std::fs::write(workspace.join("tsconfig.json"), "{}").unwrap();
    let helper = workspace.join("helpers/h1.ts");
    let source = app.join("src/App.vue");
    std::fs::write(&helper, "export const helper = 1;\n").unwrap();
    std::fs::write(&source, "<template />").unwrap();

    assert_eq!(
        classify_default_run_scope(&[helper, source], &app, &workspace),
        DefaultRunScope::Widened {
            inputs: 2,
            outside: 1
        }
    );

    let _ = std::fs::remove_dir_all(&workspace);
}

/// An ancestor program whose inputs all live in the working directory needs no
/// note: the wider root changes nothing about what is checked.
#[test]
fn ancestor_root_whose_inputs_are_all_local_is_owned() {
    let workspace = unique_case_dir("all-local");
    let app = workspace.join("nested/app");
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::write(workspace.join("tsconfig.json"), "{}").unwrap();
    let source = app.join("src/App.vue");
    std::fs::write(&source, "<template />").unwrap();

    assert_eq!(
        classify_default_run_scope(&[source], &app, &workspace),
        DefaultRunScope::Owned
    );

    let _ = std::fs::remove_dir_all(&workspace);
}

/// An empty input set is reported by the "no files found" path, not here.
#[test]
fn empty_input_set_is_owned() {
    let workspace = unique_case_dir("empty");
    let app = workspace.join("app");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::write(workspace.join("tsconfig.json"), "{}").unwrap();

    assert_eq!(
        classify_default_run_scope(&[], &app, &workspace),
        DefaultRunScope::Owned
    );

    let _ = std::fs::remove_dir_all(&workspace);
}

#[test]
fn unowned_project_error_names_both_directories_and_the_adopted_config() {
    assert_eq!(
        unowned_project_error(
            Path::new("/workspace/nested/app"),
            Path::new("/workspace"),
            Path::new("/workspace/tsconfig.json"),
            3
        ),
        "`/workspace/nested/app` has no tsconfig.json, and the nearest one above it \
         (`/workspace/tsconfig.json`) type-checks 3 files under `/workspace`, none of them inside \
         `/workspace/nested/app`. Reporting that project's result for this directory would hide \
         every error here, so nothing was checked: add a tsconfig.json to \
         `/workspace/nested/app`, pass `--tsconfig <path>`, or name the files to check."
    );
}

#[test]
fn widened_scope_note_reports_the_root_and_the_input_counts() {
    assert_eq!(
        widened_scope_note(
            Path::new("/workspace/nested/app"),
            Path::new("/workspace"),
            4,
            3
        ),
        "vize check: the project root resolved to `/workspace`, above the working directory \
         `/workspace/nested/app`; 3 of 4 checked files are outside `/workspace/nested/app`."
    );
}

fn missing_error(config: &Path, dependency: &str) -> vize_l0::String {
    cstr!(
        "Failed to resolve TypeScript configuration `{}`: Configuration error: cannot resolve tsconfig extends {dependency:?} from {}",
        config.display(),
        config.display()
    )
}

#[test]
fn missing_inherited_base_is_named_and_recreation_recovers_without_source_edit() {
    let case = tempfile::tempdir().unwrap();
    let config = case.path().join("tsconfig.json");
    let base = case.path().join("tsconfig.base.json");
    let source = case.path().join("TypedPanel.vue");
    let bytes = "<script setup lang=\"ts\">\nconst value: string = null;\n</script>\n<template><p>{{ value }}</p></template>\n";
    fs::write(&source, bytes).unwrap();
    fs::write(
        &config,
        r#"{"extends":"./tsconfig.base.json","include":["TypedPanel.vue"]}"#,
    )
    .unwrap();
    let config_before = fs::read(&config).unwrap();
    assert_eq!(
        validate_tsconfig_chain(&config).unwrap_err(),
        missing_error(&config, "./tsconfig.base.json")
    );
    fs::write(&base, r#"{"compilerOptions":{"strictNullChecks":false}}"#).unwrap();
    validate_tsconfig_chain(&config).unwrap();
    fs::remove_file(&base).unwrap();
    assert_eq!(
        validate_tsconfig_chain(&config).unwrap_err(),
        missing_error(&config, "./tsconfig.base.json")
    );
    fs::write(&base, r#"{"compilerOptions":{"strictNullChecks":true}}"#).unwrap();
    validate_tsconfig_chain(&config).unwrap();
    assert_eq!(fs::read(&source).unwrap(), bytes.as_bytes());
    assert_eq!(fs::read(&config).unwrap(), config_before);
}

#[test]
fn array_and_package_extends_use_the_same_strict_resolver() {
    let case = tempfile::tempdir().unwrap();
    let config = case.path().join("tsconfig.json");
    fs::write(case.path().join("present.json"), "{}").unwrap();
    for (content, dependency) in [
        (
            r#"{"extends":["./present.json","./missing.json"]}"#,
            "./missing.json",
        ),
        (
            r#"{"extends":"@missing/tsconfig/base.json"}"#,
            "@missing/tsconfig/base.json",
        ),
    ] {
        fs::write(&config, content).unwrap();
        assert_eq!(
            validate_tsconfig_chain(&config).unwrap_err(),
            missing_error(&config, dependency)
        );
    }
}

#[test]
fn nested_missing_dependency_names_the_selected_and_declaring_configs() {
    let case = tempfile::tempdir().unwrap();
    let config = case.path().join("tsconfig.json");
    let base = case.path().join("base.json");
    fs::write(&config, r#"{"extends":"./base.json"}"#).unwrap();
    fs::write(&base, r#"{"extends":"./missing.json"}"#).unwrap();
    assert_eq!(
        validate_tsconfig_chain(&config).unwrap_err(),
        cstr!(
            "Failed to resolve TypeScript configuration `{}`: Configuration error: cannot resolve tsconfig extends \"./missing.json\" from {}",
            config.display(),
            base.display()
        )
    );
}

#[test]
fn malformed_inherited_configuration_refuses_and_valid_repair_recovers() {
    let case = tempfile::tempdir().unwrap();
    let config = case.path().join("tsconfig.json");
    let base = case.path().join("base.json");
    fs::write(&config, r#"{"extends":"./base.json"}"#).unwrap();
    fs::write(&base, "{invalid").unwrap();
    let error = validate_tsconfig_chain(&config).unwrap_err();
    assert_eq!(
        error,
        cstr!(
            "Failed to resolve TypeScript configuration `{}`: JSON parse error: key must be a string at line 1 column 2",
            config.display()
        )
    );
    fs::write(&base, "{}").unwrap();
    validate_tsconfig_chain(&config).unwrap();
}

#[test]
fn explicit_missing_selected_config_cannot_fall_back_to_defaults() {
    let case = tempfile::tempdir().unwrap();
    let config = case.path().join("missing.json");
    assert_eq!(
        validate_tsconfig_chain(&config).unwrap_err(),
        cstr!(
            "TypeScript config file not found: `{}`; no files were checked.",
            config.display()
        )
    );
}

#[test]
fn canonical_diamond_and_cycle_policy_remains_accepted() {
    let case = tempfile::tempdir().unwrap();
    let config = case.path().join("tsconfig.json");
    fs::write(&config, r#"{"extends":["./left.json","./right.json"]}"#).unwrap();
    fs::write(
        case.path().join("left.json"),
        r#"{"extends":"./shared.json"}"#,
    )
    .unwrap();
    fs::write(
        case.path().join("right.json"),
        r#"{"extends":"./shared.json"}"#,
    )
    .unwrap();
    fs::write(case.path().join("shared.json"), "{}").unwrap();
    validate_tsconfig_chain(&config).unwrap();
    fs::write(
        case.path().join("shared.json"),
        r#"{"extends":"./tsconfig.json"}"#,
    )
    .unwrap();
    validate_tsconfig_chain(&config).unwrap();
}
