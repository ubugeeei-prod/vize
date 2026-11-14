//! #10980: a separately authored ambient composable must keep its own contract.
#![expect(
    clippy::disallowed_macros,
    reason = "complete fixture diagnostics use std strings"
)]
#![expect(clippy::disallowed_types, reason = "fixture IO uses std strings")]
#![expect(clippy::disallowed_methods, reason = "fixture IO uses std strings")]

use std::{path::Path, path::PathBuf, process::Command, process::Output};

use serde_json::{Value, json};

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const ORIGINAL: &str = include_str!(
    "../../../../../tests/_fixtures/differential/typechecker/ambient-template-ref-10980/Probe.vue.txt"
);
const AMBIENT: &str = include_str!(
    "../../../../../tests/_fixtures/differential/typechecker/ambient-template-ref-10980/global.d.ts.txt"
);
const CONFIG: &str = include_str!(
    "../../../../../tests/_fixtures/differential/typechecker/ambient-template-ref-10980/tsconfig.json.txt"
);
const FOCUS_ERROR: &str = "Property 'focus' does not exist on type '{}'.";
const VUE_AMBIENT: &str = r#"export {};
declare global {
  const useTemplateRef: typeof import("vue").useTemplateRef;
}
"#;

#[derive(Clone, Copy)]
enum Route {
    Cli,
    Mapper,
}

impl Route {
    fn name(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Mapper => "mapper",
        }
    }
}

fn config(composables: Option<&[&str]>, mapper: bool) -> Value {
    let mut config: Value = serde_json::from_str(CONFIG).unwrap();
    if let Some(composables) = composables {
        config["vueCompilerOptions"] = json!({"composables": {"useTemplateRef": composables}});
    }
    if mapper {
        config["contentMappers"] = json!([{"package": "vize", "extensions": [".vue"]}]);
    }
    config
}

fn run(route: Route, native: &Path, root: &Path) -> Output {
    match route {
        Route::Cli => Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root)
            .args([
                "check",
                "--no-config",
                "--tsconfig",
                "tsconfig.json",
                "--format",
                "json",
                "--corsa-path",
            ])
            .arg(native)
            .output()
            .unwrap(),
        Route::Mapper => check_project(native, root, "tsconfig.json"),
    }
}

fn assert_diagnostics(route: Route, output: &Output, expected: &[&str], ambient: bool) {
    match route {
        Route::Cli => {
            assert_eq!(
                output.status.code(),
                Some(i32::from(!expected.is_empty())),
                "{}",
                output_text(output)
            );
            let packet: Value = serde_json::from_slice(&output.stdout).unwrap();
            let diagnostics = expected
                .iter()
                .map(|message| format!("error:{message}"))
                .collect::<Vec<_>>();
            let mut files = vec![json!({"file":"Probe.vue", "diagnostics": diagnostics})];
            if ambient {
                files.push(json!({"file":"global.d.ts", "diagnostics":[]}));
            }
            assert_eq!(packet["files"], json!(files), "{}", output_text(output));
            assert_eq!(packet["fileCount"], files.len());
            assert_eq!(packet["errorCount"], expected.len());
            assert_eq!(packet["warningCount"], 0);
        }
        Route::Mapper => {
            assert_eq!(
                output.status.code(),
                Some(if expected.is_empty() { 0 } else { 2 })
            );
            assert_eq!(output.stderr, b"", "{}", output_text(output));
            let expected = expected
                .iter()
                .map(|message| {
                    let (position, diagnostic) = message.split_once(" [").unwrap();
                    let (line, column) = position.split_once(':').unwrap();
                    let (code, message) = diagnostic.split_once("] ").unwrap();
                    format!("Probe.vue({line},{column}): error {code}: {message}\n")
                })
                .collect::<String>();
            assert_eq!(
                output.stdout,
                expected.as_bytes(),
                "{}",
                output_text(output)
            );
        }
    }
}

fn capture(root: &Path, route: Route, case: &str, phase: &str, output: &Output) {
    let Some(directory) = std::env::var_os("VIZE_AMBIENT_TEMPLATE_REF_CAPTURE") else {
        return;
    };
    let directory = PathBuf::from(directory)
        .join(route.name())
        .join(case)
        .join(phase);
    std::fs::create_dir_all(&directory).unwrap();
    for file in ["Probe.vue", "global.d.ts", "tsconfig.json"] {
        if root.join(file).is_file() {
            std::fs::copy(root.join(file), directory.join(file)).unwrap();
        }
    }
    std::fs::write(directory.join("stdout.raw"), &output.stdout).unwrap();
    std::fs::write(directory.join("stderr.raw"), &output.stderr).unwrap();
    std::fs::write(
        directory.join("process.json"),
        json!({"route":route.name(), "cwd":root, "status":output.status.code()}).to_string(),
    )
    .unwrap();
}

#[test]
fn ambient_template_ref_retains_native_cli_and_mapper_errors_through_repairs() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping ambient composable conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(native.is_file(), "{TSGO_ENV} must name an executable");
    let typed = ORIGINAL.replace(
        "useTemplateRef('target')",
        "useTemplateRef<HTMLButtonElement>('target')",
    );
    assert_ne!(typed, ORIGINAL);
    let imported = ORIGINAL.replacen(
        "<script setup lang=\"ts\">\n",
        "<script setup lang=\"ts\">\nimport { useTemplateRef } from 'vue';\n",
        1,
    );
    for route in [Route::Cli, Route::Mapper] {
        for (case, composables, configured) in [
            ("default", None, false),
            ("disabled", Some([].as_slice()), false),
            ("unrelated", Some(["anotherRef"].as_slice()), false),
            ("configured", Some(["useTemplateRef"].as_slice()), true),
        ] {
            let project = tempfile::tempdir().unwrap();
            install_mapper_manifest(project.path());
            install_vue_package(project.path());
            let config = config(composables, matches!(route, Route::Mapper));
            std::fs::write(
                project.path().join("tsconfig.json"),
                serde_json::to_vec_pretty(&config).unwrap(),
            )
            .unwrap();
            std::fs::write(project.path().join("global.d.ts"), AMBIENT).unwrap();
            for (phase, source, expected) in [
                (
                    "original",
                    ORIGINAL,
                    vec![format!("3:32 [TS2339] {FOCUS_ERROR}")],
                ),
                ("typed-repair", typed.as_str(), vec![]),
                (
                    "restored",
                    ORIGINAL,
                    vec![format!("3:32 [TS2339] {FOCUS_ERROR}")],
                ),
            ] {
                std::fs::write(project.path().join("Probe.vue"), source).unwrap();
                let output = run(route, &native, project.path());
                capture(project.path(), route, case, phase, &output);
                let expected = if configured {
                    Vec::new()
                } else {
                    expected.iter().map(String::as_str).collect::<Vec<_>>()
                };
                assert_diagnostics(route, &output, &expected, true);
            }
            std::fs::remove_file(project.path().join("global.d.ts")).unwrap();
            let missing = run(route, &native, project.path());
            capture(project.path(), route, case, "missing-import", &missing);
            assert_diagnostics(
                route,
                &missing,
                &["2:16 [TS2304] Cannot find name 'useTemplateRef'."],
                false,
            );
            // The real imported Vue composable still owns element inference.
            std::fs::write(project.path().join("Probe.vue"), &imported).unwrap();
            let output = run(route, &native, project.path());
            capture(project.path(), route, case, "real-vue-import", &output);
            assert_diagnostics(route, &output, &[], false);
        }
    }
}

#[test]
fn vue_owned_ambient_template_ref_reaches_the_content_mapper() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping Vue ambient composable conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(native.is_file(), "{TSGO_ENV} must name an executable");
    let project = tempfile::tempdir().unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    std::fs::write(project.path().join("global.d.ts"), VUE_AMBIENT).unwrap();
    std::fs::write(project.path().join("Probe.vue"), ORIGINAL).unwrap();
    for (case, composables, expected) in [
        ("default", None, Vec::new()),
        (
            "disabled",
            Some([].as_slice()),
            vec![format!("3:32 [TS2339] {FOCUS_ERROR}")],
        ),
        (
            "configured",
            Some(["useTemplateRef"].as_slice()),
            Vec::new(),
        ),
    ] {
        std::fs::write(
            project.path().join("tsconfig.json"),
            serde_json::to_vec_pretty(&config(composables, true)).unwrap(),
        )
        .unwrap();

        let output = run(Route::Mapper, &native, project.path());
        capture(project.path(), Route::Mapper, case, "vue-owned", &output);
        let expected = expected.iter().map(String::as_str).collect::<Vec<_>>();
        assert_diagnostics(Route::Mapper, &output, &expected, true);
    }
}

#[test]
fn reported_auto_imported_dom_and_component_refs_match_explicit_imports() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping reported auto-import conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(native.is_file(), "{TSGO_ENV} must name an executable");
    let fixture = workspace_root().join(
        "tests/_fixtures/differential/typechecker/content-mapper-auto-import-template-ref-12970",
    );
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-auto-import-template-ref-")
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    for name in [
        "tsconfig.json",
        "src/auto-imports.d.ts",
        "src/child.vue",
        "src/auto-imported.vue",
        "src/imported.vue",
    ] {
        let source = fixture.join(format!("{name}.txt"));
        let target = project.path().join(name);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(source, target).unwrap();
    }

    let output = check_project(&native, project.path(), "tsconfig.json");
    assert!(output.status.success(), "{}", output_text(&output));
    assert_eq!(output.stdout, b"", "{}", output_text(&output));
    assert_eq!(output.stderr, b"", "{}", output_text(&output));

    for name in ["src/auto-imported.vue", "src/imported.vue"] {
        let path = project.path().join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        let probe = source.replace(
            "child.value?.focus()",
            "element.value?.__vizeMissingElement\nchild.value?.__vizeMissingComponent",
        );
        assert_ne!(probe, source);
        std::fs::write(path, probe).unwrap();
    }
    let output = check_project(&native, project.path(), "tsconfig.json");
    assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("error TS2339").count(), 4, "{stdout}");
    for property in ["__vizeMissingElement", "__vizeMissingComponent"] {
        assert_eq!(stdout.matches(property).count(), 2, "{stdout}");
    }
}
