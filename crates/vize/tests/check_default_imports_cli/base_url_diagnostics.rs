use super::*;

#[derive(Clone, Copy)]
enum BaseUrlKind {
    TypeScript,
    JavaScript,
    Transitive,
    Inherited,
}

impl BaseUrlKind {
    fn name(self) -> &'static str {
        match self {
            Self::TypeScript => "ts",
            Self::JavaScript => "js",
            Self::Transitive => "transitive",
            Self::Inherited => "inherited",
        }
    }
}

const KINDS: &[BaseUrlKind] = &[
    BaseUrlKind::TypeScript,
    BaseUrlKind::JavaScript,
    BaseUrlKind::Transitive,
    BaseUrlKind::Inherited,
];

#[test]
fn default_check_reports_base_url_authored_sources() {
    for &kind in KINDS {
        assert_base_url_source_is_reported(kind, false);
    }
}

#[test]
fn explicit_check_reports_base_url_authored_sources() {
    for &kind in KINDS {
        assert_base_url_source_is_reported(kind, true);
    }
}

fn assert_base_url_source_is_reported(kind: BaseUrlKind, explicit: bool) {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let mode = if explicit { "explicit" } else { "default" };
    let project_root = unique_case_dir(&format!("base-url-{}-{mode}", kind.name()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    std::fs::create_dir_all(project_root.join("support")).unwrap();
    let allow_js = matches!(kind, BaseUrlKind::JavaScript);
    let extends = matches!(kind, BaseUrlKind::Inherited);
    if extends {
        std::fs::create_dir_all(project_root.join("config")).unwrap();
        std::fs::write(
            project_root.join("config/tsconfig.base.json"),
            r#"{
  "compilerOptions": {
    "baseUrl": "..",
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  }
}"#,
        )
        .unwrap();
    }
    let config = if extends {
        r#"{
  "extends": "./config/tsconfig.base.json",
  "include": ["src/entry.ts"]
}"#
        .to_owned()
    } else {
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
    "noEmit": true
  }},
  "include": ["src/entry.ts"]
}}"#
        )
    };
    std::fs::write(project_root.join("tsconfig.json"), config).unwrap();
    let entry_specifier = match kind {
        BaseUrlKind::JavaScript => "support/invalid.js",
        BaseUrlKind::Transitive => "support/index",
        BaseUrlKind::TypeScript | BaseUrlKind::Inherited => "support/invalid",
    };
    std::fs::write(
        project_root.join("src/entry.ts"),
        format!("import '{entry_specifier}'\nexport const clean = true\n"),
    )
    .unwrap();
    if matches!(kind, BaseUrlKind::Transitive) {
        std::fs::write(
            project_root.join("support/index.ts"),
            "import './invalid'\nexport const support = true\n",
        )
        .unwrap();
    }
    let invalid_path = if allow_js {
        project_root.join("support/invalid.js")
    } else {
        project_root.join("support/invalid.ts")
    };
    let invalid_source = if allow_js {
        "/** @type {string} */\nconst invalid = 42\nvoid invalid\n"
    } else {
        "const invalid: string = 42\nvoid invalid\n"
    };
    std::fs::write(&invalid_path, invalid_source).unwrap();

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
    let expected_count = if matches!(kind, BaseUrlKind::Transitive) {
        3
    } else {
        2
    };
    assert_eq!(json["fileCount"], expected_count, "{stdout}\n{stderr}");
    assert_eq!(json["errorCount"], 1, "{stdout}\n{stderr}");
    let invalid_relative = invalid_path
        .strip_prefix(&project_root)
        .unwrap()
        .to_string_lossy();
    let invalid = json["files"]
        .as_array()
        .and_then(|files| {
            files
                .iter()
                .find(|file| file["file"] == invalid_relative.as_ref())
        })
        .unwrap_or_else(|| panic!("missing invalid source:\n{stdout}\n{stderr}"));
    assert!(
        invalid["diagnostics"]
            .as_array()
            .is_some_and(|diagnostics| diagnostics.iter().any(|diagnostic| diagnostic
                .as_str()
                .is_some_and(|diagnostic| diagnostic.contains("TS2322")))),
        "{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(project_root);
}
