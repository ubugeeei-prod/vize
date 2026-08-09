use super::*;

#[derive(Clone, Copy)]
enum PackageKind {
    RootExport,
    SubpathExport,
    LegacyTypes,
    JavaScript,
    Transitive,
    WildcardExport,
    SelfReference,
    ImportsMap,
}

impl PackageKind {
    fn name(self) -> &'static str {
        match self {
            Self::RootExport => "root-export",
            Self::SubpathExport => "subpath-export",
            Self::LegacyTypes => "legacy-types",
            Self::JavaScript => "javascript",
            Self::Transitive => "transitive",
            Self::WildcardExport => "wildcard-export",
            Self::SelfReference => "self-reference",
            Self::ImportsMap => "imports-map",
        }
    }
}

const KINDS: &[PackageKind] = &[
    PackageKind::RootExport,
    PackageKind::SubpathExport,
    PackageKind::LegacyTypes,
    PackageKind::JavaScript,
    PackageKind::Transitive,
    PackageKind::WildcardExport,
    PackageKind::SelfReference,
    PackageKind::ImportsMap,
];

#[test]
fn default_check_reports_workspace_package_sources() {
    for &kind in KINDS {
        assert_workspace_source_is_reported(kind, false);
    }
}

#[test]
fn explicit_check_reports_workspace_package_sources() {
    for &kind in KINDS {
        assert_workspace_source_is_reported(kind, true);
    }
}

fn assert_workspace_source_is_reported(kind: PackageKind, explicit: bool) {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let mode = if explicit { "explicit" } else { "default" };
    let case_root = unique_case_dir(&format!("workspace-package-{}-{mode}", kind.name()));
    let _ = std::fs::remove_dir_all(&case_root);
    let app_root = case_root.join("app");
    let package_root = case_root.join("packages/workspace-source");
    std::fs::create_dir_all(app_root.join("src")).unwrap();
    std::fs::create_dir_all(package_root.join("src")).unwrap();
    let allow_js = matches!(kind, PackageKind::JavaScript);
    std::fs::write(
        app_root.join("tsconfig.json"),
        format!(
            r##"{{
  "compilerOptions": {{
    "allowJs": {allow_js},
    "checkJs": {allow_js},
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  }},
  "include": ["src/entry.ts"]
}}"##
        ),
    )
    .unwrap();
    let entry = match kind {
        PackageKind::SubpathExport => "@scope/workspace-source/feature",
        PackageKind::WildcardExport => "@scope/workspace-source/features/alpha",
        _ => "@scope/workspace-source",
    };
    std::fs::write(
        app_root.join("src/entry.ts"),
        format!("import '{entry}'\nexport const clean = true\n"),
    )
    .unwrap();
    let exported_file = match kind {
        PackageKind::SubpathExport => "src/feature.ts",
        PackageKind::JavaScript => "src/index.js",
        PackageKind::WildcardExport => "src/features/alpha.ts",
        _ => "src/index.ts",
    };
    let package_json = if matches!(kind, PackageKind::LegacyTypes) {
        r#"{
  "name": "@scope/workspace-source",
  "types": "./src/index.ts"
}"#
        .to_owned()
    } else {
        let target = format!("./{exported_file}");
        format!(
            r##"{{
  "name": "@scope/workspace-source",
  "exports": {{
    ".": {{ "types": "{target}", "default": "{target}" }},
    "./feature": {{ "types": "./src/feature.ts", "default": "./src/feature.ts" }},
    "./features/*": {{ "types": "./src/features/*.ts", "default": "./src/features/*.ts" }},
    "./internal": {{ "types": "./src/internal.ts", "default": "./src/internal.ts" }}
  }},
  "imports": {{
    "#internal": {{ "types": "./src/internal.ts", "default": "./src/internal.ts" }}
  }}
}}"##
        )
    };
    std::fs::write(package_root.join("package.json"), package_json).unwrap();
    if matches!(kind, PackageKind::Transitive) {
        std::fs::write(
            package_root.join("src/index.ts"),
            "import './invalid'\nexport const support = true\n",
        )
        .unwrap();
    } else if matches!(kind, PackageKind::SelfReference) {
        std::fs::write(
            package_root.join("src/index.ts"),
            "import '@scope/workspace-source/internal'\nexport const support = true\n",
        )
        .unwrap();
    } else if matches!(kind, PackageKind::ImportsMap) {
        std::fs::write(
            package_root.join("src/index.ts"),
            "import '#internal'\nexport const support = true\n",
        )
        .unwrap();
    }
    let invalid_path = match kind {
        PackageKind::Transitive => package_root.join("src/invalid.ts"),
        PackageKind::SelfReference | PackageKind::ImportsMap => {
            package_root.join("src/internal.ts")
        }
        _ => package_root.join(exported_file),
    };
    let invalid_source = if allow_js {
        "/** @type {string} */\nconst invalid = 42\nvoid invalid\n"
    } else {
        "const invalid: string = 42\nvoid invalid\n"
    };
    std::fs::create_dir_all(invalid_path.parent().unwrap()).unwrap();
    std::fs::write(&invalid_path, invalid_source).unwrap();
    link_workspace_package(
        &package_root,
        &app_root.join("node_modules/@scope/workspace-source"),
    );

    let mut args = vec!["check", "--format", "json"];
    if explicit {
        args.push("src/entry.ts");
    }
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&app_root)
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
    let expected_count = if matches!(
        kind,
        PackageKind::Transitive | PackageKind::SelfReference | PackageKind::ImportsMap
    ) {
        3
    } else {
        2
    };
    assert_eq!(json["fileCount"], expected_count, "{stdout}\n{stderr}");
    assert_eq!(json["errorCount"], 1, "{stdout}\n{stderr}");
    let invalid_display = invalid_path.to_string_lossy();
    let invalid = json["files"]
        .as_array()
        .and_then(|files| {
            files
                .iter()
                .find(|file| file["file"] == invalid_display.as_ref())
        })
        .unwrap_or_else(|| panic!("missing {invalid_display}:\n{stdout}\n{stderr}"));
    assert!(
        invalid["diagnostics"]
            .as_array()
            .is_some_and(|diagnostics| diagnostics.iter().any(|diagnostic| diagnostic
                .as_str()
                .is_some_and(|diagnostic| diagnostic.contains("TS2322")))),
        "{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(case_root);
}

fn link_workspace_package(source: &Path, target: &Path) {
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}
