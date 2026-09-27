#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::path::{Path, PathBuf};
use std::process::Command;

/// A `{ get, set }` computed is a writable instance property. Assigning to it
/// from an Options API template (`@input="ratio = $event"`) type-checks in
/// vue-tsc; Vize used to declare every computed as `const` and report
/// `TS2588 Cannot assign to 'ratio' because it is a constant`.
#[test]
fn check_options_api_writable_computed_assignment_passes() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let Some(vue_package) = workspace_vue_package() else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "VIZE_TEST_REQUIRE_TSGO requires a workspace Vue package for the computed CLI fixture"
        );
        eprintln!("skipping writable computed CLI fixture: workspace Vue package missing");
        return;
    };
    let project_root = create_cli_project(&vue_package);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/WritableComputed.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    let diagnostics = json["files"]
        .as_array()
        .expect("check JSON must contain files")
        .iter()
        .flat_map(|file| {
            file["diagnostics"]
                .as_array()
                .expect("each file must contain diagnostics")
        })
        .map(|diagnostic| {
            diagnostic
                .as_str()
                .expect("diagnostics must be strings")
                .to_owned()
        })
        .collect::<Vec<_>>();

    // The complete oracle keeps the getter-only assignment, its source
    // position and message, while rejecting any writable-computed error.
    let expected: Vec<String> = serde_json::from_str(include_str!(
        "../../../tests/fixtures/typechecker/options-api-writable-computed/expected-diagnostics.json"
    ))
    .unwrap();
    assert_eq!(
        diagnostics, expected,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(output.status.code(), Some(1));

    let _ = std::fs::remove_dir_all(&project_root);
}

/// A throwaway project under `target/` with the fixture component and a
/// symlink to the workspace `node_modules` (for `vue`).
fn create_cli_project(vue_package: &Path) -> PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join(format!(
            "options-api-writable-computed-{}",
            std::process::id()
        ));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root, vue_package).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        include_str!(
            "../../../tests/fixtures/typechecker/options-api-writable-computed/tsconfig.json"
        ),
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/WritableComputed.vue"),
        include_str!(
            "../../../tests/fixtures/typechecker/options-api-writable-computed/WritableComputed.vue"
        ),
    )
    .unwrap();
    project_root
}

/// The repository root, two levels above this crate.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

/// Locate Vue where this workspace actually declares it, including the tests
/// package. A dependency-free local run can defer; required CI cannot.
fn workspace_vue_package() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.join("node_modules/vue"),
        root.join("tests/node_modules/vue"),
        root.join("playground/node_modules/vue"),
        root.join("examples/vite-musea/node_modules/vue"),
        root.join("examples/jsx-tsx/node_modules/vue"),
        root.join("npm/framework/nuxt/node_modules/vue"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

/// Match existing Canon CLI fixtures: link Vue and its namespace into the
/// isolated project instead of assuming the workspace root depends on Vue.
fn link_workspace_vue(project_root: &Path, vue_package: &Path) -> std::io::Result<()> {
    let workspace_node_modules = vue_package.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace Vue package has no node_modules parent",
        )
    })?;
    let target = project_root.join("node_modules");
    std::fs::create_dir_all(&target)?;
    symlink_path(vue_package, &target.join("vue"))?;
    let vue_namespace = workspace_node_modules.join("@vue");
    if vue_namespace.exists() {
        symlink_path(&vue_namespace, &target.join("@vue"))?;
    }
    Ok(())
}

/// The Corsa binary to check with: `CORSA_PATH` when set, else the workspace
/// `tsgo` shim. Returned absolute, as the CLI runs from the fixture project.
fn resolve_test_corsa_path() -> Option<String> {
    if let Some(path) = std::env::var_os("CORSA_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            let path = path.canonicalize().unwrap_or(path);
            return Some(path.display().to_string());
        }
    }
    let workspace_root = workspace_root();
    [workspace_root.join("node_modules/.bin/tsgo")]
        .into_iter()
        .find(|candidate| candidate.exists())
        .map(|candidate| candidate.display().to_string())
}

/// Create a directory symlink on either platform.
fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}
