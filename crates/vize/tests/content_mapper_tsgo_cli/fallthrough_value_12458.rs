//! #12458: a fallthrough attribute value error reaches the Content Mapper host
//! with the code, message and span `vize check` reports.

use std::{fs, path::PathBuf, process::Command};

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};
use vize_l0::{String as CompactString, cstr};

const FIELD: &str = "<script setup lang=\"ts\">\ndefineProps<{ label: string }>()\n</script>\n<template><input :aria-label=\"label\" /></template>\n";
const APP: &str = "<script setup lang=\"ts\">\nimport Field from './Field.vue'\n</script>\n<template>\n  <Field label=\"x\" :maxlength=\"{}\" />\n  <Field :label=\"1\" :tabindex=\"true\" :aria-busy=\"1\" />\n</template>\n";
const CONFIG: &str = r#"{
  "compilerOptions": { "strict": true, "module": "ESNext", "target": "ESNext", "moduleResolution": "Bundler", "noEmit": true, "skipLibCheck": true, "types": [] },
  "vueCompilerOptions": { "strictTemplates": true },
  "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
  "include": ["src/**/*.vue"]
}
"#;
/// One owner each: the original fallthrough value, a declared prop, a
/// literal fallthrough value and an ARIA attr.
const EXPECTED: [(u32, u32, &str); 4] = [
    (
        5,
        21,
        "Type '{}' is not assignable to type 'Numberish | undefined'.",
    ),
    (6, 11, "Type 'number' is not assignable to type 'string'."),
    (
        6,
        22,
        "Type 'true' is not assignable to type 'Numberish | undefined'.",
    ),
    (
        6,
        39,
        "Type '1' is not assignable to type 'Booleanish | undefined'.",
    ),
];

#[test]
fn standard_tsgo_reports_fallthrough_values_like_vize_check() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("fallthrough-value-12458-")
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    fs::write(project.path().join("src/Field.vue"), FIELD).unwrap();
    fs::write(project.path().join("src/App.vue"), APP).unwrap();

    let output = check_project(&tsgo, project.path(), "tsconfig.json");
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let mut mapped = stdout
        .lines()
        .filter(|line| line.contains(": error TS"))
        .collect::<Vec<_>>();
    mapped.sort_unstable();
    let expected = EXPECTED
        .iter()
        .map(|(line, column, message)| {
            cstr!("src/App.vue({line},{column}): error TS2322: {message}")
        })
        .collect::<Vec<_>>();
    assert_eq!(mapped, expected, "{}", output_text(&output));

    let Some(corsa) = crate::support::corsa_path::resolve(workspace_root()) else {
        eprintln!("skipping the vize check side: no Corsa executable resolved");
        return;
    };
    let batch = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project.path())
        .env("CORSA_PATH", corsa)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "src",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&batch.stdout)
        .unwrap_or_else(|error| panic!("{error}\n{}", output_text(&batch)));
    let mut checked = report["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap())
        .map(|diagnostic| {
            let text = diagnostic.as_str().unwrap();
            CompactString::from(text.split_once(" (source: ").map_or(text, |(head, _)| head))
        })
        .collect::<Vec<_>>();
    checked.sort_unstable();
    let expected = EXPECTED
        .iter()
        .map(|(line, column, message)| cstr!("error:{line}:{column} [TS2322] {message}"))
        .collect::<Vec<_>>();
    assert_eq!(checked, expected, "{}", output_text(&batch));
}
