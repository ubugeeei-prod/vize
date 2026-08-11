//! Regression for importer-scoped package identity (#4002).

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::{path::Path, process::Command};

#[path = "support/corsa_path.rs"]
mod corsa_path;
#[path = "check_importer_scoped_package_cli/project_references.rs"]
mod project_references;

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn write_file(root: &Path, path: &str, content: &str) {
    let file = root.join(path);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file, content).unwrap();
}

fn resolve_test_corsa_path() -> Option<String> {
    corsa_path::resolve(workspace_root())
}

fn run_check(project: &Path, corsa: &str, declaration: bool) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command.current_dir(project).env("CORSA_PATH", corsa).args([
        "check",
        "--no-config",
        "--format",
        "json",
    ]);
    if declaration {
        command.args(["--declaration", "--declaration-dir", "types"]);
    }
    command.output().unwrap()
}

fn run_explicit(project: &Path, corsa: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project)
        .env("CORSA_PATH", corsa)
        .args([
            "check",
            "apps/alpha/src/entry.ts",
            "apps/bravo/src/entry.ts",
            "--no-config",
            "--format",
            "json",
        ])
        .output()
        .unwrap()
}

fn install_package(project: &Path, app: &str, prop: &str, type_name: &str) {
    let package = format!("apps/{app}/node_modules/@scope/ui");
    write_file(
        project,
        &format!("{package}/package.json"),
        &format!(
            r##"{{
  "name": "@scope/ui",
  "version": "{type_name}",
  "exports": {{
    ".": {{ "vize-test": "./src/index.ts", "types": "./src/Fallback.vue", "default": "./src/Fallback.vue" }},
    "./feature/*": {{ "types": "./src/features/*.vue", "default": "./src/features/*.vue" }},
    "./feature/special/*": {{ "types": "./src/special/*.vue", "default": "./src/special/*.vue" }}
  }},
  "imports": {{ "#internal": "@scope/internal" }}
}}
"##
        ),
    );
    write_file(
        project,
        &format!("{package}/src/index.ts"),
        "export { default } from './Conditional.vue'\nexport * from './Conditional.vue'\n",
    );
    write_file(
        project,
        &format!("apps/{app}/node_modules/@scope/internal/package.json"),
        "{\n  \"name\": \"@scope/internal\",\n  \"exports\": \"./src/Internal.vue\"\n}\n",
    );
    write_file(
        project,
        &format!("apps/{app}/node_modules/@scope/internal/src/Internal.vue"),
        &format!("<script setup lang=\"ts\">defineProps<{{ {prop}: {type_name} }}>()</script>\n"),
    );
    write_file(
        project,
        &format!("{package}/src/Conditional.vue"),
        &format!(
            r##"<script setup lang="ts">
import Internal from "#internal"
type InternalProps = InstanceType<typeof Internal>["$props"]
const privateIdentity: InternalProps = {{ {prop}: {} }}
void privateIdentity
defineProps<{{ {prop}: {type_name} }}>()
</script>
"##,
            if type_name == "string" { "'ok'" } else { "1" }
        ),
    );
    write_file(
        project,
        &format!("{package}/src/Fallback.vue"),
        "<script setup lang=\"ts\">defineProps<{ fallbackOnly: Date }>()</script>\n",
    );
    write_file(
        project,
        &format!("{package}/src/features/Card.vue"),
        &format!(
            "<script setup lang=\"ts\">defineProps<{{ {prop}Feature: boolean }}>()</script>\n"
        ),
    );
    write_file(
        project,
        &format!("{package}/src/features/special/Card.vue"),
        "<script setup lang=\"ts\">defineProps<{ wrongPattern: never }>()</script>\n",
    );
    write_file(
        project,
        &format!("{package}/src/special/Card.vue"),
        &format!(
            "<script setup lang=\"ts\">defineProps<{{ {prop}Special: boolean }}>()</script>\n"
        ),
    );
}

#[test]
fn duplicate_package_versions_and_private_imports_stay_importer_scoped() {
    let Some(corsa) = resolve_test_corsa_path() else {
        return;
    };
    let project = workspace_root()
        .join("target/vize-tests/tests")
        .join(format!("importer-package-identity-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project);
    write_file(
        &project,
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "skipLibCheck": true,
    "declaration": true,
    "declarationMap": true,
    "customConditions": ["vize-test"]
  },
  "include": ["apps/*/src/**/*.ts"]
}
"#,
    );
    install_package(&project, "alpha", "alpha", "string");
    install_package(&project, "bravo", "bravo", "number");
    write_file(
        &project,
        "apps/alpha/src/entry.ts",
        r#"import Widget from "@scope/ui"
import Feature from "@scope/ui/feature/Card"
import Special from "@scope/ui/feature/special/Card"
type Props = InstanceType<typeof Widget>["$props"]
type FeatureProps = InstanceType<typeof Feature>["$props"]
type SpecialProps = InstanceType<typeof Special>["$props"]
export const alphaProps: Props = { alpha: "ok" }
export const alphaFeature: FeatureProps = { alphaFeature: true }
export const alphaSpecial: SpecialProps = { alphaSpecial: true }
"#,
    );
    write_file(
        &project,
        "apps/bravo/src/entry.ts",
        r#"import Widget from "@scope/ui"
import Feature from "@scope/ui/feature/Card"
import Special from "@scope/ui/feature/special/Card"
type Props = InstanceType<typeof Widget>["$props"]
type FeatureProps = InstanceType<typeof Feature>["$props"]
type SpecialProps = InstanceType<typeof Special>["$props"]
export const bravoProps: Props = { bravo: 1 }
export const bravoFeature: FeatureProps = { bravoFeature: true }
export const bravoSpecial: SpecialProps = { bravoSpecial: true }
"#,
    );
    let authored_before = ["alpha", "bravo"].map(|app| {
        let path = project.join(format!("apps/{app}/src/entry.ts"));
        (path.clone(), std::fs::read(&path).unwrap())
    });

    let explicit = run_explicit(&project, &corsa);
    assert!(
        explicit.status.success(),
        "explicit importer routes disagreed with default check:\n{}\n{}",
        String::from_utf8_lossy(&explicit.stdout),
        String::from_utf8_lossy(&explicit.stderr)
    );

    let clean = run_check(&project, &corsa, true);
    let clean_stdout = String::from_utf8(clean.stdout).unwrap();
    let clean_stderr = String::from_utf8(clean.stderr).unwrap();
    assert!(
        clean.status.success(),
        "duplicate versions or #imports collapsed:\n{clean_stdout}\n{clean_stderr}"
    );
    for output in [&clean_stdout, &clean_stderr] {
        assert!(!output.contains("__vize_external__"), "{output}");
        assert!(!output.contains(".vize/canon"), "{output}");
    }
    for (app, declaration) in [
        ("alpha", project.join("types/alpha/src/entry.d.ts")),
        ("bravo", project.join("types/bravo/src/entry.d.ts")),
    ] {
        let source = std::fs::read_to_string(&declaration)
            .unwrap_or_else(|error| panic!("{}: {error}", declaration.display()));
        assert!(source.contains("@scope/ui"), "{source}");
        assert!(
            !source.contains("__vize") && !source.contains(".vize/canon"),
            "{source}"
        );
        let map = std::fs::read_to_string(declaration.with_extension("ts.map"))
            .expect("declaration map should be emitted");
        assert!(
            !map.contains("__vize") && !map.contains(".vize/canon"),
            "{map}"
        );
        let map: serde_json::Value = serde_json::from_str(&map).unwrap();
        let sources = map["sources"].as_array().unwrap();
        assert!(
            sources
                .iter()
                .any(|source| source.as_str().is_some_and(|source| {
                    source
                        .replace('\\', "/")
                        .ends_with(&format!("apps/{app}/src/entry.ts"))
                })),
            "declaration map did not point back to authored {app} entry: {map}"
        );
    }
    for (path, bytes) in &authored_before {
        assert_eq!(std::fs::read(path).unwrap(), *bytes, "{}", path.display());
    }

    write_file(
        &project,
        "apps/bravo/src/entry.ts",
        r#"import Widget from "@scope/ui"
import Feature from "@scope/ui/feature/Card"
import Special from "@scope/ui/feature/special/Card"
type Props = InstanceType<typeof Widget>["$props"]
type FeatureProps = InstanceType<typeof Feature>["$props"]
type SpecialProps = InstanceType<typeof Special>["$props"]
export const bravoProps: Props = { alpha: "wrong" }
export const bravoFeature: FeatureProps = { bravoFeature: true }
export const bravoSpecial: SpecialProps = { bravoSpecial: true }
"#,
    );
    let broken = run_check(&project, &corsa, false);
    let stdout = String::from_utf8(broken.stdout).unwrap();
    let stderr = String::from_utf8(broken.stderr).unwrap();
    assert!(
        !broken.status.success(),
        "wrong package API passed:\n{stdout}\n{stderr}"
    );
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let files = report["files"].as_array().unwrap();
    let diagnostics_for = |suffix: &str| {
        files
            .iter()
            .find(|file| {
                file["file"]
                    .as_str()
                    .is_some_and(|path| path.ends_with(suffix))
            })
            .and_then(|file| file["diagnostics"].as_array())
            .unwrap()
    };
    assert!(
        diagnostics_for("apps/bravo/src/entry.ts")
            .iter()
            .any(|diagnostic| diagnostic
                .as_str()
                .is_some_and(|text| text.contains("TS2353"))),
        "{stdout}\n{stderr}"
    );
    assert!(
        diagnostics_for("apps/alpha/src/entry.ts").is_empty(),
        "{stdout}\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project);
}
