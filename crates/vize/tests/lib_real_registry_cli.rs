#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! `vize lib pull` against the real `@vizejs/ui` and `@vizejs/composable`
//! registries, freshly emitted from this checkout by their Node build scripts.
//!
//! Proves pulled code is self-contained: every relative import of every pulled
//! file resolves inside the pulled tree, and every pulled `.vue` file compiles
//! with `vize build` (which resolves imported prop types across files).
//! Skipped when `node` cannot run the TypeScript build scripts.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Emit `registry/` for one package into `out` (a copy, never the checkout).
fn build_registry(package: &str, out: &Path) -> Option<PathBuf> {
    let package_dir = repo_root().join(package);
    let status = Command::new("node")
        .arg("scripts/build-source-registry.ts")
        .current_dir(&package_dir)
        .output()
        .ok()?;
    if !status.status.success() {
        eprintln!(
            "skipping: node could not build the {package} registry: {}",
            String::from_utf8_lossy(&status.stderr)
        );
        return None;
    }
    let source = package_dir.join("registry");
    copy_dir(&source, out);
    Some(out.to_path_buf())
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn vize(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

fn assert_ok(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn files_under(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files_under(&path, found);
        } else {
            found.push(path);
        }
    }
}

/// Relative specifiers in `from "./x"`, `import "./x"`, and `import("./x")`.
fn relative_specifiers(source: &str) -> Vec<String> {
    let mut specifiers = Vec::new();
    for quote in ['"', '\''] {
        for (index, _) in source.match_indices(quote) {
            let Some(rest) = source.get(index + 1..) else {
                continue;
            };
            if !(rest.starts_with("./") || rest.starts_with("../")) {
                continue;
            }
            let before = source.get(..index).unwrap_or_default().trim_end();
            if !(before.ends_with("from")
                || before.ends_with("import")
                || before.ends_with("import("))
            {
                continue;
            }
            if let Some(end) = rest.find(quote) {
                specifiers.push(rest.get(..end).unwrap_or_default().to_owned());
            }
        }
    }
    specifiers
}

#[test]
fn pulls_real_items_that_compile_and_resolve_on_their_own() {
    let temp = tempfile::tempdir().unwrap();
    let Some(ui) = build_registry("npm/ui", &temp.path().join("ui-registry")) else {
        return;
    };
    let Some(composable) =
        build_registry("npm/compose/core", &temp.path().join("composable-registry"))
    else {
        return;
    };
    let root = temp.path().join("app");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{ "dependencies": { "vue": "^3.5.0" } }"#,
    )
    .unwrap();

    let pulled = vize(
        &root,
        &[
            "lib",
            "--registry",
            ui.to_str().unwrap(),
            "--registry",
            composable.to_str().unwrap(),
            "pull",
            "switch",
            "use-storage",
        ],
    );
    assert_ok(&pulled);

    let lock: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("vize-lib.lock.json")).unwrap()).unwrap();
    let names: Vec<&str> = lock["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["name"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"switch") && names.contains(&"use-storage"),
        "{names:?}"
    );
    assert!(
        names.contains(&"controllable-state"),
        "switch pulls its foundations: {names:?}"
    );

    let mut files = Vec::new();
    files_under(&root.join("src"), &mut files);
    let mut vue_files = Vec::new();
    for file in &files {
        let source = fs::read_to_string(file).unwrap();
        for specifier in relative_specifiers(&source) {
            let target = file.parent().unwrap().join(&specifier);
            assert!(
                target.is_file(),
                "{} imports missing {specifier}",
                file.display()
            );
        }
        if file.extension().is_some_and(|extension| extension == "vue") {
            vue_files.push(
                file.strip_prefix(&root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
            );
        }
    }
    assert!(!vue_files.is_empty());

    for vue_file in &vue_files {
        assert_ok(&vize(
            &root,
            &[
                "build",
                "--format",
                "js",
                vue_file.as_str(),
                "--output",
                "dist",
            ],
        ));
    }

    let status = vize(
        &root,
        &[
            "lib",
            "--registry",
            ui.to_str().unwrap(),
            "--registry",
            composable.to_str().unwrap(),
            "--json",
            "status",
        ],
    );
    assert_ok(&status);
    let status: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    for item in status["items"].as_array().unwrap() {
        assert_eq!(item["state"], "clean", "{item}");
        assert_eq!(item["upstream"], "up-to-date", "{item}");
    }
}
