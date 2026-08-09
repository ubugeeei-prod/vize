#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

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
