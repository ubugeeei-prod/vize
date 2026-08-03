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
    let root = workspace_root();
    let sibling_cache = root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache.display().to_string());
    }

    let workspace_bin = root.join("node_modules/.bin/tsgo");
    workspace_bin
        .exists()
        .then(|| workspace_bin.display().to_string())
}

#[test]
fn path_aliases_do_not_resolve_project_json_to_canon_control_files() {
    let Some(corsa_path) = resolve_test_corsa_path() else {
        return;
    };

    let project_root = workspace_root()
        .join("target/vize-tests/tests")
        .join(format!("canon-path-alias-json-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    write_file(
        &project_root,
        "package.json",
        r#"{ "name": "alias-json-app", "version": "1.2.3" }"#,
    );
    write_file(
        &project_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "resolveJsonModule": true,
    "strict": true,
    "noEmit": true,
    "paths": {
      "~/*": ["./*"],
      "@/*": ["./*"]
    }
  }
}"#,
    );
    write_file(
        &project_root,
        "repro.ts",
        r#"import { version as tildeVersion } from "~/package.json";
import { version as atVersion } from "@/package.json";

export const versions: [string, string] = [tildeVersion, atVersion];
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "repro.ts", "--no-config", "--format", "json"])
        .output()
        .unwrap();

    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "project JSON aliases resolved to Canon control files:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stdout.contains("node_modules/.vize/canon/package"),
        "diagnostics must not resolve aliases to Canon's package boundary:\n{stdout}\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["warningCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(json["fileCount"], 1, "{stdout}\n{stderr}");

    let _ = std::fs::remove_dir_all(&project_root);
}
