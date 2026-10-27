#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use crate::support::{corsa_path, corsa_requirement, vue_stub};
use std::{path::Path, process::Command};

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file_path, content).unwrap();
}

fn resolve_test_corsa_path() -> Option<String> {
    let workspace_root = workspace_root();
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache.display().to_string());
    }

    let workspace_bin = workspace_root.join("node_modules/.bin/tsgo");
    workspace_bin
        .exists()
        .then(|| workspace_bin.display().to_string())
}

#[test]
fn check_from_package_cwd_keeps_local_tsconfig_for_external_relative_imports() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };

    let workspace = workspace_root()
        .join("target/vize-tests/tests")
        .join(format!(
            "package-cwd-external-relative-import-{}",
            std::process::id()
        ));
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(&workspace).unwrap();
    write_file(
        &workspace,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "target": "ES2018",
    "module": "ESNext",
    "moduleResolution": "Node",
    "baseUrl": ".",
    "strict": true,
    "noEmit": true
  },
  "exclude": ["node_modules", "pkg"]
}"#,
    );
    write_file(
        &workspace,
        "shared/types.ts",
        "export type Shared = { id: number };\n",
    );

    let package_root = workspace.join("pkg");
    write_file(&package_root, "package.json", r#"{ "name": "pkg" }"#);
    write_file(
        &package_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*.ts"]
}"#,
    );
    write_file(
        &package_root,
        "node_modules/exports-only/package.json",
        r#"{
  "name": "exports-only",
  "version": "1.0.0",
  "type": "module",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js"
    }
  }
}"#,
    );
    write_file(
        &package_root,
        "node_modules/exports-only/dist/index.d.ts",
        "export declare function hello(): string;\n",
    );
    write_file(
        &package_root,
        "src/main.ts",
        r#"import { hello } from "exports-only";
import type { Shared } from "../../shared/types";

export const greeting: string = hello();
export const shared: Shared = { id: 1 };
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&package_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--no-config", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "package-local tsconfig was not authoritative:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains(&format!(
            "Building Corsa virtual project for 2 files under {}...",
            package_root.display()
        )),
        "the invocation project root should remain package-local:\n{stderr}"
    );
    assert!(
        !stdout.contains("TS2307")
            && !stdout.contains("TS5102")
            && !stdout.contains("TS5108")
            && !stdout.contains(&workspace.join("tsconfig.json").display().to_string()),
        "the outer tsconfig must not affect package diagnostics:\n{stdout}\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["warningCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["fileCount"], 2, "{stdout}\n{stderr}");
    let files = json["files"]
        .as_array()
        .expect("JSON output should contain files");
    assert!(
        files
            .iter()
            .any(|file| file["file"].as_str().is_some_and(|path| {
                path.ends_with("/shared/types.ts") && Path::new(path).is_absolute()
            }))
            && files.iter().any(|file| file["file"] == "src/main.ts"),
        "{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&workspace);
}

/// Sibling SFCs and inherited hoisted JSX inputs must retain package storage (#10934).
#[test]
fn relative_sibling_sfc_keeps_package_storage_and_exact_repeated_diagnostics() {
    let Some(corsa) = corsa_requirement::required_or_skip(corsa_path::resolve(workspace_root()))
    else {
        return;
    };
    let directory = tempfile::tempdir().unwrap();
    let workspace = directory.path().canonicalize().unwrap();
    vue_stub::install_vue_jsx_type_stub(&workspace);
    let fixture_dir = workspace_root()
        .join("tests/_fixtures/differential/typechecker/relative-sibling-storage-10934");
    let pack: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture_dir.join("cases.json")).unwrap()).unwrap();
    for input in pack["cases"][0]["inputs"].as_array().unwrap() {
        write_file(
            &workspace,
            input["file"].as_str().unwrap(),
            &std::fs::read_to_string(fixture_dir.join(input["source"].as_str().unwrap())).unwrap(),
        );
    }
    let app = workspace.join("apps/app");
    let virtual_root = vize_canon::persistent_batch_virtual_root(
        &workspace,
        &app,
        Some(&app.join("tsconfig.json")),
    );
    let jsx = workspace.join("node_modules/.pnpm/vue@3.4.23/node_modules/vue/jsx.d.ts");
    let original_jsx = std::fs::read(&jsx).unwrap();
    let expected = serde_json::json!([[
        "src/entry.ts",
        "error:2:31 [TS2322] Type 'string' is not assignable to type 'number'."
    ]]);
    let mut persisted = Vec::new();
    let mut build_info = Vec::new();
    for run in 0..3 {
        if run == 2 {
            write_file(
                &workspace,
                "apps/app/src/entry.ts",
                &std::fs::read_to_string(fixture_dir.join("repaired/entry.ts.txt")).unwrap(),
            );
        }
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&app)
            .env("CORSA_PATH", &corsa)
            .args(["check", "src/*.ts", "--no-config", "--format", "json"])
            .args(["--tsconfig", "tsconfig.json"])
            .output()
            .unwrap();
        super::check_outside_import_types_cli::capture(
            &app,
            Path::new(&corsa),
            &format!("sibling-{run}"),
            &[],
            &output,
        );
        let stdout = String::from_utf8(output.stdout).unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(
            output.status.code(),
            Some(i32::from(run != 2)),
            "run {run}:\n{stdout}\n{stderr}"
        );
        let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let diagnostics = report["files"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|file| {
                file["diagnostics"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|message| serde_json::json!([file["file"], message]))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            serde_json::json!(diagnostics),
            if run == 2 {
                serde_json::json!([])
            } else {
                expected.clone()
            },
            "run {run}:\n{stdout}\n{stderr}"
        );
        assert_eq!(report["errorCount"], usize::from(run != 2));
        assert_eq!(report["warningCount"], 0);
        assert_eq!(report["programs"][0]["root"], ".", "{stdout}");
        assert!(
            stderr.contains(&format!(" under {}...", app.display())),
            "the requested package must retain its storage root:\n{stderr}"
        );
        assert!(
            virtual_root.join("apps/app/src/entry.ts").is_file(),
            "the stable package mirror must survive process exit: {}",
            virtual_root.display()
        );
        let mut owned = Vec::new();
        collect_owned_files(&virtual_root, &mut owned);
        let authored_json = workspace.join("packages/ui/src/data.json");
        let mirrored_json = owned
            .iter()
            .filter(|path| *path == &virtual_root.join("packages/ui/src/data.json"))
            .collect::<Vec<_>>();
        assert_eq!(
            mirrored_json.len(),
            1,
            "JSON must have one owned external mirror"
        );
        assert_eq!(
            std::fs::read(mirrored_json[0]).unwrap(),
            std::fs::read(&authored_json).unwrap(),
            "the mirrored JSON must preserve its exact bytes"
        );
        let mut current_build_info = owned
            .iter()
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "tsbuildinfo")
            })
            .cloned()
            .collect::<Vec<_>>();
        current_build_info.sort();
        assert!(
            !current_build_info.is_empty(),
            "incremental state must survive process exit"
        );
        if run == 0 {
            build_info = current_build_info;
            for path in owned
                .into_iter()
                .filter(|path| path.ends_with("Shared.vue.ts") || path.ends_with("src/model.ts"))
            {
                persisted.push((
                    path.clone(),
                    std::fs::metadata(path).unwrap().modified().unwrap(),
                ));
            }
            assert_eq!(
                persisted.len(),
                2,
                "both sides of the package boundary must be mirrored"
            );
            assert!(
                persisted
                    .iter()
                    .any(|(path, _)| path == &virtual_root.join("packages/ui/src/Shared.vue.ts")),
                "the sibling SFC must retain its authored workspace geometry"
            );
        } else {
            assert_eq!(
                current_build_info, build_info,
                "run {run} must reuse the same incremental namespace"
            );
            for (path, modified) in &persisted {
                assert_eq!(
                    std::fs::metadata(path).unwrap().modified().unwrap(),
                    *modified,
                    "unchanged mirror must not be rewritten on run {run}: {}",
                    path.display()
                );
            }
        }
        assert_eq!(
            std::fs::read(&jsx).unwrap(),
            original_jsx,
            "installed declarations are never writable project storage"
        );
    }
    std::fs::remove_dir_all(virtual_root).unwrap();
}
/// Inspect only owned files; package links may point at shared installations.
fn collect_owned_files(root: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            collect_owned_files(&entry.path(), files);
        } else if kind.is_file() {
            files.push(entry.path());
        }
    }
}
