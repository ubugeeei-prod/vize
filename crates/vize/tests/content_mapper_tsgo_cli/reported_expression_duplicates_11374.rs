//! Original #11374 diagnostics and independent repairs through the real mapper host.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::TSGO_ENV;
use super::support::{check_project, install_mapper_manifest, install_vue_package, workspace_root};

const SOURCES: [&str; 4] = [
    "Child.vue",
    "MultilineProp.vue",
    "MultilineVIf.vue",
    "NestedVIf.vue",
];

fn input() -> Value {
    serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/reported-expression-duplicates-11374/input.json"
    )))
    .unwrap()
}

fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

#[test]
fn authored_errors_survive_without_synthetic_expression_echoes() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping original expression duplicate controls: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_absolute() && tsgo.is_file());
    let fixture = input();
    let expected = fixture["expectedDiagnostics"].as_str().unwrap();
    let original_lines = expected.lines().collect::<Vec<_>>();
    assert_eq!(original_lines.len(), 3);
    let cases = [
        ("original", Vec::new(), false),
        ("repair-prop", vec!["MultilineProp.vue"], false),
        ("repair-guard", vec!["MultilineVIf.vue"], false),
        ("repair-nested", vec!["NestedVIf.vue"], false),
        (
            "repair-all",
            vec!["MultilineProp.vue", "MultilineVIf.vue", "NestedVIf.vue"],
            false,
        ),
        (
            "independent-script-error",
            vec!["MultilineProp.vue", "MultilineVIf.vue", "NestedVIf.vue"],
            true,
        ),
        ("restored-original", Vec::new(), false),
    ];
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let mut observations = Vec::new();
    let mut verdicts = Vec::new();
    for (id, repairs, independent_error) in cases {
        let project = tempfile::Builder::new()
            .prefix("reported-expression-duplicates-11374-")
            .tempdir_in(&cases_root)
            .unwrap();
        let root = project.path();
        write(
            root,
            "package.json",
            fixture["packageJson"].as_str().unwrap(),
        );
        write(root, "tsconfig.json", fixture["tsconfig"].as_str().unwrap());
        let mut sources = serde_json::Map::new();
        for name in SOURCES {
            let original = fixture[name].as_str().unwrap();
            let source = if repairs.contains(&name) {
                assert_eq!(original.matches("items[0].label").count(), 1);
                original.replace("items[0].label", "items[0]!.label")
            } else {
                original.to_owned()
            };
            write(root, &format!("src/{name}"), &source);
            sources.insert(name.to_owned(), json!(source));
        }
        if independent_error {
            let source = "const wrong: string = 1; void wrong;\n";
            write(root, "src/Control.ts", source);
            sources.insert("Control.ts".to_owned(), json!(source));
        }
        install_mapper_manifest(root);
        install_vue_package(root);
        let vue_manifest =
            std::fs::canonicalize(root.join("node_modules/vue/package.json")).unwrap();
        let output = check_project(&tsgo, root, "tsconfig.json");
        let expected_stdout = if independent_error {
            "src/Control.ts(1,7): error TS2322: Type 'number' is not assignable to type 'string'.\n"
                .to_owned()
        } else {
            original_lines
                .iter()
                .filter(|line| {
                    !repairs
                        .iter()
                        .any(|name| line.starts_with(&format!("src/{name}(")))
                })
                .map(|line| format!("{line}\n"))
                .collect::<String>()
        };
        let expected_status = if expected_stdout.is_empty() { 0 } else { 2 };
        observations.push(json!({
            "id": id,
            "cwd": root,
            "sources": sources,
            "config": fixture["tsconfig"],
            "package": fixture["packageJson"],
            "mapperManifest": std::fs::read(root.join("node_modules/vize/package.json")).unwrap(),
            "vueManifestPath": vue_manifest,
            "vueManifest": std::fs::read(&vue_manifest).unwrap(),
            "host": tsgo,
            "argv": ["--runExternalCode", "--noEmit", "-p", "tsconfig.json", "--pretty", "false"],
            "status": output.status.code(),
            "osStatus": output.status.to_string(),
            "stdout": output.stdout,
            "stderr": output.stderr,
            "expectedStatus": expected_status,
            "expectedStdout": expected_stdout,
        }));
        verdicts.push((id, output, expected_status, expected_stdout));
    }
    // Preserve every complete process observation even when the original case fails.
    eprintln!(
        "reported-expression-duplicates-11374: {}",
        serde_json::to_string(&observations).unwrap()
    );
    for (id, output, status, stdout) in verdicts {
        assert_eq!(output.status.code(), Some(status), "{id}");
        assert_eq!(output.stdout, stdout.as_bytes(), "{id}");
        assert_eq!(output.stderr, b"", "{id}");
    }
}

mod receiver_contracts;
