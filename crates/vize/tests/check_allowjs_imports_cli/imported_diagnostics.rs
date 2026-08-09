use super::*;

const IMPORT_CASES: &[(&str, &str, &str)] = &[
    ("js-explicit", "js", "../support/invalid.js"),
    ("js-extensionless", "js", "../support/invalid"),
    ("mjs", "mjs", "../support/invalid.mjs"),
    ("cjs", "cjs", "../support/invalid.cjs"),
    ("jsx", "jsx", "../support/invalid.jsx"),
];

#[test]
fn default_check_reports_imported_javascript_family_diagnostics() {
    for &(case, extension, specifier) in IMPORT_CASES {
        assert_imported_javascript_is_reported(case, extension, specifier, None);
    }
}

#[test]
fn explicit_check_reports_imported_javascript_family_diagnostics() {
    for &(case, extension, specifier) in IMPORT_CASES {
        assert_imported_javascript_is_reported(case, extension, specifier, Some("src/entry.ts"));
    }
}

#[test]
fn explicit_check_keeps_ambient_only_module_javascript_non_reporting() {
    let Some(corsa_path) = required_corsa_path() else {
        return;
    };
    let project_root = unique_case_dir("ambient-only-module-js");
    let _ = std::fs::remove_dir_all(&project_root);
    write(
        &project_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*.ts", "types/**/*.d.ts"]
}"#,
    );
    write(&project_root, "src/entry.ts", "export const clean = true\n");
    write(
        &project_root,
        "types/context.d.ts",
        "import '../support/invalid.mjs'\nimport '../support/invalid.cjs'\nexport {}\n",
    );
    for extension in ["mjs", "cjs"] {
        write(
            &project_root,
            &format!("support/invalid.{extension}"),
            "/** @type {string} */\nconst invalid = 42\nvoid invalid\n",
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
            "src/entry.ts",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stdout}\n{stderr}");
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["fileCount"], 1, "{stdout}\n{stderr}");
    assert_eq!(json["errorCount"], 0, "{stdout}\n{stderr}");
    assert_eq!(
        json["files"][0]["file"], "src/entry.ts",
        "{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(project_root);
}

fn assert_imported_javascript_is_reported(
    case: &str,
    extension: &str,
    specifier: &str,
    explicit: Option<&str>,
) {
    let Some(corsa_path) = required_corsa_path() else {
        return;
    };
    let mode = if explicit.is_some() {
        "explicit"
    } else {
        "default"
    };
    let project_root = unique_case_dir(&format!("imported-{case}-{mode}"));
    let _ = std::fs::remove_dir_all(&project_root);
    write(
        &project_root,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "allowJs": true,
    "checkJs": true,
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "noEmit": true
  },
  "include": ["src/entry.ts"]
}"#,
    );
    write(
        &project_root,
        "src/entry.ts",
        &format!("import '{specifier}'\nexport const clean = true\n"),
    );
    let imported_file = format!("support/invalid.{extension}");
    write(
        &project_root,
        &imported_file,
        "/** @type {string} */\nconst invalid = 42\nvoid invalid\n",
    );

    let mut args = vec!["check", "--no-config", "--tsconfig", "tsconfig.json"];
    args.extend(explicit);
    args.extend(["--format", "json"]);
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args(args)
        .output()
        .unwrap();
    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "{case}/{mode}:\n{stdout}\n{stderr}"
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["fileCount"], 2, "{case}/{mode}:\n{stdout}\n{stderr}");
    assert_eq!(json["errorCount"], 1, "{case}/{mode}:\n{stdout}\n{stderr}");
    let imported = json["files"]
        .as_array()
        .and_then(|files| files.iter().find(|file| file["file"] == imported_file))
        .unwrap_or_else(|| panic!("missing {imported_file}:\n{stdout}\n{stderr}"));
    assert!(
        imported["diagnostics"]
            .as_array()
            .is_some_and(|diagnostics| diagnostics.iter().any(|diagnostic| {
                diagnostic
                    .as_str()
                    .is_some_and(|diagnostic| diagnostic.contains("TS2322"))
            })),
        "{case}/{mode}:\n{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(project_root);
}
