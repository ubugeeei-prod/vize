use super::*;

#[derive(Clone, Copy)]
enum ImportKind {
    AliasTypeScript,
    AliasJavaScript,
    AliasTransitive,
    AbsoluteTypeScript,
}

impl ImportKind {
    fn name(self) -> &'static str {
        match self {
            Self::AliasTypeScript => "alias-ts",
            Self::AliasJavaScript => "alias-js",
            Self::AliasTransitive => "alias-transitive",
            Self::AbsoluteTypeScript => "absolute-ts",
        }
    }

    fn imported_file(self) -> &'static str {
        match self {
            Self::AliasJavaScript => "support/invalid.js",
            _ => "support/invalid.ts",
        }
    }
}

const IMPORT_KINDS: &[ImportKind] = &[
    ImportKind::AliasTypeScript,
    ImportKind::AliasJavaScript,
    ImportKind::AliasTransitive,
    ImportKind::AbsoluteTypeScript,
];

#[test]
fn default_check_reports_aliased_and_absolute_authored_sources() {
    for &kind in IMPORT_KINDS {
        assert_authored_import_is_reported(kind, false);
    }
}

#[test]
fn explicit_check_reports_aliased_and_absolute_authored_sources() {
    for &kind in IMPORT_KINDS {
        assert_authored_import_is_reported(kind, true);
    }
}

#[test]
fn sharded_check_reports_an_aliased_source_owned_by_a_nonzero_shard() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("alias-sharded-nonzero-owner");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    std::fs::create_dir_all(project_root.join("support")).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "baseUrl": ".",
    "paths": { "@/*": ["*"] },
    "noEmit": true
  },
  "include": ["src/*.ts"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/heavy.ts"),
        format!("export const payload = '{}'\n", "x".repeat(16_384)),
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/alias.ts"),
        format!(
            "import '@/support/invalid'\n/* {} */\nexport const alias = true\n",
            "y".repeat(12_000)
        ),
    )
    .unwrap();
    std::fs::write(
        project_root.join("support/invalid.ts"),
        "const invalid: string = 42\nvoid invalid\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--format", "json", "--servers", "2"])
        .output()
        .unwrap();
    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["fileCount"], 3, "{stdout}\n{stderr}");
    assert_eq!(json["errorCount"], 1, "{stdout}\n{stderr}");

    let _ = std::fs::remove_dir_all(project_root);
}

fn assert_authored_import_is_reported(kind: ImportKind, explicit: bool) {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let mode = if explicit { "explicit" } else { "default" };
    let project_root = unique_case_dir(&format!("{}-{mode}", kind.name()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    std::fs::create_dir_all(project_root.join("support")).unwrap();
    let allow_js = matches!(kind, ImportKind::AliasJavaScript);
    std::fs::write(
        project_root.join("tsconfig.json"),
        format!(
            r#"{{
  "compilerOptions": {{
    "allowJs": {allow_js},
    "checkJs": {allow_js},
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "baseUrl": ".",
    "paths": {{ "@/*": ["*"] }},
    "noEmit": true
  }},
  "include": ["src/entry.ts"]
}}"#
        ),
    )
    .unwrap();
    let imported_path = if matches!(kind, ImportKind::AbsoluteTypeScript) {
        project_root.with_file_name(format!(
            "{}-external.ts",
            project_root.file_name().unwrap().to_string_lossy()
        ))
    } else {
        project_root.join(kind.imported_file())
    };
    let specifier = match kind {
        ImportKind::AbsoluteTypeScript => imported_path.to_string_lossy().into_owned(),
        ImportKind::AliasTransitive => "@/support/index".into(),
        ImportKind::AliasJavaScript => "@/support/invalid.js".into(),
        ImportKind::AliasTypeScript => "@/support/invalid".into(),
    };
    std::fs::write(
        project_root.join("src/entry.ts"),
        format!("import '{specifier}'\nexport const clean = true\n"),
    )
    .unwrap();
    if matches!(kind, ImportKind::AliasTransitive) {
        std::fs::write(
            project_root.join("support/index.ts"),
            "import './invalid'\nexport const support = true\n",
        )
        .unwrap();
    }
    let invalid_source = if allow_js {
        "/** @type {string} */\nconst invalid = 42\nvoid invalid\n"
    } else {
        "const invalid: string = 42\nvoid invalid\n"
    };
    std::fs::write(&imported_path, invalid_source).unwrap();

    let mut args = vec!["check", "--format", "json"];
    if explicit {
        args.push("src/entry.ts");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(args)
        .output()
        .unwrap();
    let stdout = std::string::String::from_utf8(output.stdout).unwrap();
    let stderr = std::string::String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}/{mode}:\n{stdout}\n{stderr}",
        kind.name()
    );
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let expected_count = if matches!(kind, ImportKind::AliasTransitive) {
        3
    } else {
        2
    };
    assert_eq!(
        json["fileCount"],
        expected_count,
        "{}/{mode}:\n{stdout}\n{stderr}",
        kind.name()
    );
    assert_eq!(
        json["errorCount"],
        1,
        "{}/{mode}:\n{stdout}\n{stderr}",
        kind.name()
    );
    let expected_file = imported_path
        .strip_prefix(&project_root)
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|_| imported_path.to_string_lossy().into_owned());
    let imported = json["files"]
        .as_array()
        .and_then(|files| files.iter().find(|file| file["file"] == expected_file))
        .unwrap_or_else(|| panic!("missing {expected_file}:\n{stdout}\n{stderr}"));
    assert!(
        imported["diagnostics"]
            .as_array()
            .is_some_and(|diagnostics| diagnostics.iter().any(|diagnostic| diagnostic
                .as_str()
                .is_some_and(|diagnostic| diagnostic.contains("TS2322")))),
        "{}/{mode}:\n{stdout}\n{stderr}",
        kind.name()
    );

    let _ = std::fs::remove_dir_all(project_root);
    if matches!(kind, ImportKind::AbsoluteTypeScript) {
        let _ = std::fs::remove_file(imported_path);
    }
}
