use super::*;

#[derive(Clone, Copy)]
enum VueExportKind {
    Root,
    Barrel,
    Subpath,
    Wildcard,
    SelfReference,
    ImportsMap,
}

impl VueExportKind {
    fn name(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Barrel => "barrel",
            Self::Subpath => "subpath",
            Self::Wildcard => "wildcard",
            Self::SelfReference => "self-reference",
            Self::ImportsMap => "imports-map",
        }
    }

    fn entry_specifier(self) -> &'static str {
        match self {
            Self::Subpath => "@scope/workspace-vue/feature",
            Self::Barrel => "@scope/workspace-vue/barrel",
            Self::Wildcard => "@scope/workspace-vue/features/alpha",
            Self::Root | Self::SelfReference | Self::ImportsMap => "@scope/workspace-vue",
        }
    }

    fn entry_file(self) -> &'static str {
        match self {
            Self::Wildcard => "src/entry.tsx",
            Self::Root | Self::Barrel | Self::Subpath | Self::SelfReference | Self::ImportsMap => {
                "src/entry.ts"
            }
        }
    }

    fn diagnosed_file(self) -> &'static str {
        match self {
            Self::Barrel | Self::Subpath | Self::SelfReference | Self::ImportsMap => {
                "src/Feature.vue"
            }
            Self::Wildcard => "src/features/alpha.vue",
            Self::Root => "src/Root.vue",
        }
    }

    fn expected_file_count(self) -> u64 {
        match self {
            Self::Barrel | Self::SelfReference | Self::ImportsMap => 3,
            Self::Root | Self::Subpath | Self::Wildcard => 2,
        }
    }
}

const KINDS: &[VueExportKind] = &[
    VueExportKind::Root,
    VueExportKind::Barrel,
    VueExportKind::Subpath,
    VueExportKind::Wildcard,
    VueExportKind::SelfReference,
    VueExportKind::ImportsMap,
];

#[test]
fn default_check_typechecks_workspace_package_vue_exports() {
    for &kind in KINDS {
        assert_workspace_vue_export_is_typechecked(kind, false);
    }
}

#[test]
fn explicit_check_typechecks_workspace_package_vue_exports() {
    for &kind in KINDS {
        assert_workspace_vue_export_is_typechecked(kind, true);
    }
}

fn assert_workspace_vue_export_is_typechecked(kind: VueExportKind, explicit: bool) {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let mode = if explicit { "explicit" } else { "default" };
    let case_name = cstr!("workspace-vue-{}-{mode}", kind.name());
    let case_root = unique_case_dir(case_name.as_str());
    let _ = std::fs::remove_dir_all(&case_root);
    let app_root = case_root.join("app");
    let package_root = case_root.join("packages/workspace-vue");
    std::fs::create_dir_all(app_root.join("src")).unwrap();
    std::fs::create_dir_all(package_root.join("src/features")).unwrap();

    std::fs::write(
        app_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/entry.ts", "src/entry.tsx"]
}"#,
    )
    .unwrap();
    std::fs::write(
        app_root.join(kind.entry_file()),
        cstr!(
            r#"import Widget from '{}'

type IsAny<T> = 0 extends 1 & T ? true : false
const componentMustBeTyped: IsAny<typeof Widget> = false
const props: InstanceType<typeof Widget>['$props'] = {{ count: 1 }}
void componentMustBeTyped
void props
"#,
            kind.entry_specifier()
        )
        .as_str(),
    )
    .unwrap();
    std::fs::write(
        package_root.join("package.json"),
        r##"{
  "name": "@scope/workspace-vue",
  "exports": {
    ".": { "types": "./src/Root.vue", "default": "./src/Root.vue" },
    "./barrel": { "types": "./src/index.ts", "default": "./src/index.ts" },
    "./feature": { "types": "./src/Feature.vue", "default": "./src/Feature.vue" },
    "./features/*": { "types": "./src/features/*.vue", "default": "./src/features/*.vue" }
  },
  "imports": {
    "#feature": { "types": "./src/Feature.vue", "default": "./src/Feature.vue" }
  }
}"##,
    )
    .unwrap();

    let root_source = match kind {
        VueExportKind::SelfReference => {
            component_source(Some("@scope/workspace-vue/feature"), false)
        }
        VueExportKind::ImportsMap => component_source(Some("#feature"), false),
        VueExportKind::Root => component_source(None, true),
        VueExportKind::Barrel | VueExportKind::Subpath | VueExportKind::Wildcard => {
            component_source(None, false)
        }
    };
    std::fs::write(package_root.join("src/Root.vue"), root_source).unwrap();
    std::fs::write(
        package_root.join("src/index.ts"),
        "export { default } from './Feature.vue'\n",
    )
    .unwrap();
    std::fs::write(
        package_root.join("src/Feature.vue"),
        component_source(
            None,
            matches!(
                kind,
                VueExportKind::Barrel
                    | VueExportKind::Subpath
                    | VueExportKind::SelfReference
                    | VueExportKind::ImportsMap
            ),
        ),
    )
    .unwrap();
    std::fs::write(
        package_root.join("src/features/alpha.vue"),
        component_source(None, matches!(kind, VueExportKind::Wildcard)),
    )
    .unwrap();
    link_workspace_vue_package(
        &package_root,
        &app_root.join("node_modules/@scope/workspace-vue"),
    );

    let mut args = vec!["check", "--format", "json"];
    if explicit {
        args.push(kind.entry_file());
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
    assert_eq!(
        json["fileCount"].as_u64(),
        Some(kind.expected_file_count()),
        "{}/{mode}:\n{stdout}\n{stderr}",
        kind.name()
    );
    assert_eq!(json["errorCount"], 2, "{stdout}\n{stderr}");

    let entry = file_result(&json, &app_root.join(kind.entry_file()), &stdout, &stderr);
    assert_eq!(
        entry["diagnostics"],
        serde_json::json!([]),
        "the package import must resolve to a typed component:\n{stdout}\n{stderr}"
    );

    let diagnosed_path = package_root.join(kind.diagnosed_file());
    let diagnosed = file_result(&json, &diagnosed_path, &stdout, &stderr);
    let diagnostics = diagnosed["diagnostics"].as_array().unwrap();
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .as_str()
            .is_some_and(|message| message.contains("error:3:7 [TS2322]"))),
        "script diagnostic must map to the original SFC:\n{stdout}\n{stderr}"
    );
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .as_str()
            .is_some_and(|message| message.contains("error:6:")
                && message.contains("[TS2339]")
                && message.contains("toUpperCase"))),
        "template diagnostic must map to the original SFC:\n{stdout}\n{stderr}"
    );
    assert!(
        all_diagnostics(&json)
            .all(|message| !message.contains("TS2307") && !message.contains("TS2882")),
        "workspace package .vue exports must not produce module-resolution errors:\n{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(case_root);
}

fn component_source(import: Option<&str>, invalid: bool) -> vize_s0::String {
    let import = import
        .map(|specifier| cstr!("import Nested from '{specifier}'\nvoid Nested\n"))
        .unwrap_or_default();
    let invalid = if invalid {
        "const invalid: string = 42\nconst templateValue = 1\n"
    } else {
        "const invalid: string = 'valid'\nconst templateValue = 'valid'\n"
    };
    cstr!(
        "<script setup lang=\"ts\">\ndefineProps<{{ count: number }}>()\n{invalid}{import}</script>\n<template>{{{{ templateValue.toUpperCase() }}}}</template>\n"
    )
}

fn file_result<'a>(
    json: &'a serde_json::Value,
    path: &Path,
    stdout: &str,
    stderr: &str,
) -> &'a serde_json::Value {
    let path = canonicalize_non_verbatim(path).display().to_string();
    json["files"]
        .as_array()
        .and_then(|files| {
            files.iter().find(|file| {
                file["file"].as_str().is_some_and(|reported| {
                    reported == path || Path::new(path.as_str()).ends_with(reported)
                })
            })
        })
        .unwrap_or_else(|| panic!("missing {path}:\n{stdout}\n{stderr}"))
}

fn all_diagnostics(json: &serde_json::Value) -> impl Iterator<Item = &str> {
    json["files"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|file| file["diagnostics"].as_array().into_iter().flatten())
        .filter_map(serde_json::Value::as_str)
}

fn link_workspace_vue_package(source: &Path, target: &Path) {
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}
