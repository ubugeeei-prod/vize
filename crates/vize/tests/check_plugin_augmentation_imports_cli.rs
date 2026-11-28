#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! An app entry that registers a package plugin (`dayjs.extend(relativeTime)`)
//! loads the plugin's `declare module 'dayjs'` augmentation for every file,
//! as under `tsc -p`, even when the checked files never import the entry.

use std::{path::Path, process::Command};

use crate::support::{corsa_requirement, vue_stub};

const FIXTURE: &str = "tests/_fixtures/differential/typechecker/plugin-augmentation-imports";

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn resolve_test_corsa_path() -> Option<std::path::PathBuf> {
    let sibling_cache = workspace_root().parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache);
    }
    let workspace_bin = workspace_root().join("node_modules/.bin/tsgo");
    workspace_bin.exists().then_some(workspace_bin)
}

fn copy_fixture(root: &Path, target: &str, source: &str) {
    let path = root.join(target);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::copy(workspace_root().join(FIXTURE).join(source), path).unwrap();
}

#[test]
fn explicit_check_sees_plugin_augmentations_registered_by_the_entry() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = workspace_root()
        .join("target/vize-tests/tests")
        .join(format!(
            "plugin-augmentation-imports-{}",
            std::process::id()
        ));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    vue_stub::install_vue_jsx_type_stub(&project_root);
    std::fs::write(
        project_root.join("package.json"),
        r#"{ "name": "plugin-augmentation-imports", "private": true, "type": "module" }"#,
    )
    .unwrap();
    for (target, source) in [
        ("tsconfig.json", "tsconfig.json.txt"),
        ("src/main.ts", "main.ts.txt"),
        ("src/App.vue", "App.vue.txt"),
        ("src/Misuse.vue", "Misuse.vue.txt"),
        ("node_modules/dayjs/package.json", "dayjs-package.json.txt"),
        ("node_modules/dayjs/index.d.ts", "dayjs-index.d.ts.txt"),
        (
            "node_modules/dayjs/plugin/relativeTime.d.ts",
            "dayjs-relativeTime.d.ts.txt",
        ),
    ] {
        copy_fixture(&project_root, target, source);
    }

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/App.vue",
            "src/Misuse.vue",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("{error}\nstdout:\n{stdout}\nstderr:\n{stderr}"));
    let diagnostics = json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| {
            let name = file["file"].as_str().unwrap().to_string();
            file["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |diagnostic| format!("{name} {}", diagnostic.as_str().unwrap()))
        })
        .collect::<Vec<_>>();
    // vue-tsc reports the misuse against the augmented signature, and no
    // TS2339 for the member the plugin adds.
    assert_eq!(
        diagnostics,
        vec![
            "src/Misuse.vue error:4:39 [TS2345] Argument of type '\"yes\"' is not assignable to parameter of type 'boolean | undefined'."
        ],
        "stderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
