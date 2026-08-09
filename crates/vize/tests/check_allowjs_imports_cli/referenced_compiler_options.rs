use super::*;

#[test]
fn referenced_projects_preserve_each_child_compiler_options() {
    let Some(corsa_path) = required_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("referenced-compiler-options");
    let _ = std::fs::remove_dir_all(&project_root);

    write(
        &project_root,
        "tsconfig.json",
        r#"{
  "files": [],
  "references": [
    { "path": "./packages/strict" },
    { "path": "./packages/indexed" },
    { "path": "./packages/loose" }
  ]
}"#,
    );
    write_child_config(&project_root, "strict", "");
    write_child_config(
        &project_root,
        "indexed",
        r#"    "noUncheckedIndexedAccess": true,"#,
    );
    write_child_config(&project_root, "loose", "");
    write(
        &project_root,
        "packages/strict/src/implicit-any.ts",
        "export const identity = value => value;\n",
    );
    let indexed_source = "const values: string[] = [];\nexport const first: string = values[0];\n";
    write(
        &project_root,
        "packages/indexed/src/unchecked-index.ts",
        indexed_source,
    );
    write(
        &project_root,
        "packages/loose/src/allowed-index.ts",
        indexed_source,
    );

    assert_program_diagnostics(&project_root, &corsa_path, &[]);
    assert_program_diagnostics(
        &project_root,
        &corsa_path,
        &[
            "packages/strict/src/implicit-any.ts",
            "packages/indexed/src/unchecked-index.ts",
            "packages/loose/src/allowed-index.ts",
        ],
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn referenced_projects_emit_declarations_to_each_child_output() {
    let Some(corsa_path) = required_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("referenced-declarations");
    let _ = std::fs::remove_dir_all(&project_root);
    write(
        &project_root,
        "tsconfig.json",
        r#"{
  "files": [],
  "references": [
    { "path": "./packages/a" },
    { "path": "./packages/b" }
  ]
}"#,
    );
    for package in ["a", "b"] {
        write(
            &project_root,
            &format!("packages/{package}/tsconfig.json"),
            r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "declaration": true,
    "declarationDir": "types"
  },
  "include": ["src/**/*.ts"]
}"#,
        );
        write(
            &project_root,
            &format!("packages/{package}/src/{package}.ts"),
            &format!(
                "export interface {}Value {{ value: string }}\n",
                package.to_uppercase()
            ),
        );
    }

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
            "--declaration",
        ])
        .output()
        .unwrap();
    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    assert_eq!(json["fileCount"], serde_json::json!(2), "{stdout}");
    assert_eq!(
        json["declarations"],
        serde_json::json!(["packages/a/types/a.d.ts", "packages/b/types/b.d.ts"]),
        "{stdout}"
    );
    assert!(project_root.join("packages/a/types/a.d.ts").is_file());
    assert!(project_root.join("packages/b/types/b.d.ts").is_file());

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn empty_allowjs_sibling_does_not_enable_javascript_in_a_deny_program() {
    let Some(corsa_path) = required_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("referenced-empty-allowjs-sibling");
    let _ = std::fs::remove_dir_all(&project_root);
    write(
        &project_root,
        "tsconfig.json",
        r#"{
  "files": [],
  "references": [
    { "path": "./packages/deny" },
    { "path": "./packages/allow" }
  ]
}"#,
    );
    for (package, allow_js) in [("deny", false), ("allow", true)] {
        write(
            &project_root,
            &format!("packages/{package}/tsconfig.json"),
            &format!(
                r#"{{
  "compilerOptions": {{
    "allowJs": {allow_js},
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  }},
  "include": ["src/**/*"]
}}"#
            ),
        );
    }
    write(
        &project_root,
        "packages/deny/src/entry.ts",
        "import './invalid.js'\nexport const clean = true\n",
    );
    write(
        &project_root,
        "packages/deny/src/invalid.js",
        "/** @type {string} */\nconst invalid = 42\nvoid invalid\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--no-config",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["fileCount"], 1, "{stdout}\n{stderr}");
    assert!(
        json["files"].as_array().is_some_and(|files| files
            .iter()
            .all(|file| file["file"] != "packages/deny/src/invalid.js")),
        "{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(project_root);
}

fn write_child_config(project_root: &Path, package: &str, extra_option: &str) {
    write(
        project_root,
        &format!("packages/{package}/tsconfig.json"),
        &format!(
            r#"{{
  "compilerOptions": {{
    "strict": true,
{extra_option}
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  }},
  "include": ["src/**/*.ts"]
}}"#
        ),
    );
}

fn assert_program_diagnostics(project_root: &Path, corsa_path: &Path, patterns: &[&str]) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--no-config", "--tsconfig", "tsconfig.json"])
        .args(patterns)
        .args(["--format", "json"]);
    let output = command.output().unwrap();
    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("invalid JSON ({error}):\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    assert_eq!(json["errorCount"], serde_json::json!(2), "{stdout}");
    assert_eq!(json["fileCount"], serde_json::json!(3), "{stdout}");

    let files = json["files"].as_array().unwrap();
    assert_file_has_code(files, "implicit-any.ts", "TS7006");
    assert_file_has_code(files, "unchecked-index.ts", "TS2322");
    let loose = files
        .iter()
        .find(|file| {
            file["file"]
                .as_str()
                .is_some_and(|path| path.ends_with("allowed-index.ts"))
        })
        .unwrap();
    assert_eq!(loose["diagnostics"], serde_json::json!([]), "{stdout}");
}

fn assert_file_has_code(files: &[serde_json::Value], name: &str, code: &str) {
    let file = files
        .iter()
        .find(|file| {
            file["file"]
                .as_str()
                .is_some_and(|path| path.ends_with(name))
        })
        .unwrap_or_else(|| panic!("missing {name}: {files:#?}"));
    assert!(
        file["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(serde_json::Value::as_str)
            .any(|diagnostic| diagnostic.contains(code)),
        "missing {code}: {file:#?}"
    );
}
