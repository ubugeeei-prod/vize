//! Original #11359 and one-change controls on the real Content Mapper host.

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, workspace_root,
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};
use vize_l0::cstr;

#[test]
fn resolved_model_props_preserve_authored_contract_and_assignment_errors() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        assert!(
            std::env::var_os("VIZE_REPORTED_MODEL_CAPTURE").is_none(),
            "reported model capture requires the Content Mapper host"
        );
        eprintln!("skipping exact reported model controls: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file());
    let fixture: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/reported-model-props-11359/input.json"
    )))
    .unwrap();
    assert_eq!(fixture["issue"], 11359);
    let rows = fixture["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 7);
    let cases_root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&cases_root).unwrap();
    let capture = std::env::var_os("VIZE_REPORTED_MODEL_CAPTURE")
        .map(PathBuf::from)
        .unwrap_or_else(|| cases_root.join("reported-model-props-11359-capture"));
    fs::create_dir_all(&capture).unwrap();
    let mut results = Vec::new();
    for row in rows {
        let id = row["id"].as_str().unwrap();
        let project = tempfile::Builder::new()
            .prefix("reported-model-props-")
            .tempdir_in(&cases_root)
            .unwrap();
        fs::create_dir_all(project.path().join("src")).unwrap();
        fs::write(
            project.path().join("package.json"),
            fixture["originalPackageJson"].as_str().unwrap(),
        )
        .unwrap();
        fs::write(
            project.path().join("tsconfig.json"),
            fixture["tsconfig"].as_str().unwrap(),
        )
        .unwrap();
        let authored = row["source"].as_str().unwrap();
        let source_path = project.path().join("src/Probe.vue");
        fs::write(&source_path, authored).unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        let output = check_project(&tsgo, project.path(), "tsconfig.json");
        let transform =
            vize_canon::generate_vue_content_mapper_transform(&source_path, authored).unwrap();
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        let codes: Vec<u32> = stdout
            .lines()
            .filter_map(|line| {
                let (_, tail) = line.split_once(": error TS")?;
                let code = tail.split_once(':')?.0;
                code.parse().ok()
            })
            .collect();
        let observed = json!({"id": id, "input": row, "tsconfig": fixture["tsconfig"],
            "declaredPackage": fixture["originalPackageJson"], "actualMapperManifest": fs::read_to_string(project.path().join("node_modules/vize/package.json")).unwrap(),
            "host": tsgo, "actualVueManifest": fs::read_to_string(project.path().join("node_modules/vue/package.json")).unwrap(),
            "arguments": ["--runExternalCode", "--noEmit", "-p", "tsconfig.json", "--pretty", "false"],
            "status": output.status.code(), "stdout": output.stdout, "stderr": output.stderr,
            "standaloneCompanionTransform": transform, "codes": codes});
        fs::write(
            capture.join(cstr!("{id}.json").as_str()),
            serde_json::to_vec_pretty(&observed).unwrap(),
        )
        .unwrap();
        results.push((id, row["codes"].clone(), codes, output));
    }
    // Preserve every original, isolation, negative and repair before any verdict.
    for (id, expected, codes, output) in results {
        assert_eq!(
            json!(codes),
            expected,
            "{id}: {}",
            super::output_text(&output)
        );
        assert_eq!(
            output.status.code(),
            Some(if codes.is_empty() { 0 } else { 2 }),
            "{id}"
        );
        assert!(
            output.stderr.is_empty(),
            "{id}: {}",
            super::output_text(&output)
        );
        if codes.is_empty() {
            assert!(output.stdout.is_empty(), "{id}");
        }
    }
}
