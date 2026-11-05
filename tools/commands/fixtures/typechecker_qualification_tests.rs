//! Exercise the production subprocess, payload validator and status together.

use super::*;

#[test]
fn configuration_inputs_cannot_qualify_the_matrix_producer() {
    let receipt: Value = serde_json::from_str(include_str!(
        "../../../tests/tooling/fixtures/typechecker-configuration/receipt.json"
    ))
    .unwrap();
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let fixture = root.join("fixture");
    let reports = root.join("reports");
    fs::create_dir_all(fixture.join("src")).unwrap();
    fs::create_dir_all(&reports).unwrap();
    fs::write(fixture.join("src/App.vue"), "<template />\n").unwrap();
    fs::write(fixture.join("tsconfig.json"), "{}\n").unwrap();
    let executable = root.join("vize.mjs");
    let project = json!({
        "id": "configuration", "fixturePath": "fixture", "expectedVueFileCount": 1,
        "vueGlobs": ["src/**/*.vue"], "tsconfig": "tsconfig.json"
    });
    let launch = Launch {
        command: "node".to_string(),
        prefix: vec![executable.to_string_lossy().into_owned()],
        label: "fixture-vize".to_string(),
    };
    let args = parse_args(Vec::new()).unwrap();
    let authored = "error:1:1 [TS2322] Type 'string' is not assignable to type 'number'.";
    for (configuration, authored_error, expected_status) in [
        (true, false, "configuration"),
        (true, true, "configuration"),
        (false, true, "findings"),
        (false, false, "ok"),
    ] {
        let mut files = vec![json!({
            "file": "src/App.vue",
            "diagnostics": if authored_error { vec![authored] } else { Vec::new() }
        })];
        if configuration {
            files.push(receipt["diagnosticFile"].clone());
        }
        let error_count = if configuration { 5 } else { 0 } + usize::from(authored_error);
        let exit_code = i32::from(error_count != 0);
        let parsed = json!({
            "errorCount": error_count, "warningCount": 0, "fileCount": 1, "files": files
        });
        let stdout = serde_json::to_string(&parsed).unwrap();
        fs::write(
            &executable,
            format!(
                "process.stdout.write({}); process.exit({exit_code});\n",
                serde_json::to_string(&stdout).unwrap()
            ),
        )
        .unwrap();
        let run = run_tool(root, &project, "typechecker", &args, &launch, &reports).unwrap();
        let payload = common::read_json(reports.join("configuration-typechecker.json")).unwrap();
        let inputs = vec!["src/App.vue".to_string()];
        let coverage =
            validate_typechecker_output(&project, &parsed, exit_code, Some(&inputs), &inputs)
                .unwrap();
        assert_eq!(
            payload,
            json!({
                "schema": "vize.fixtureToolRun", "version": 1, "project": "configuration",
                "tool": "typechecker", "exitCode": exit_code, "stdout": stdout, "stderr": "",
                "parsed": parsed, "typecheckerCoverage": coverage
            })
        );
        assert_eq!(
            json!({"status": run["status"], "exitCode": run["exitCode"], "fileCount": run["fileCount"], "coverage": run["coverage"]}),
            json!({"status": expected_status, "exitCode": exit_code, "fileCount": 1, "coverage": summarize_typechecker_coverage(&coverage).unwrap()})
        );
    }
}

#[test]
fn typechecker_qualification_uses_error_codes_instead_of_diagnostic_messages() {
    for (file, diagnostic, expected) in [
        (
            "configs/build.jsonc",
            "error:1:1 [TS5023] Unknown compiler option 'missing'.",
            "configuration",
        ),
        (
            "src/App.vue",
            "error:1:1 [TS2307] Cannot find module '#imports'.",
            "configuration",
        ),
        (
            "src/App.vue",
            "error:1:1 [TS6305] Output file has not been built from source file.",
            "configuration",
        ),
        (
            "src/App.vue",
            "error:1:1 [TS2322] Type 'TS2688' is not assignable to type 'number'.",
            "findings",
        ),
        (
            "tsconfig.json",
            "warning:1:1 [TS2688] A warning does not prevent semantic checking.",
            "findings",
        ),
        (
            "src/App.vue",
            "hint:1:1 [TS2688] A hint does not prevent semantic checking.",
            "findings",
        ),
    ] {
        let packet = json!({"files": [{"file": file, "diagnostics": [diagnostic]}]});
        assert_eq!(
            typechecker_qualification::typechecker_run_status(1, &packet),
            expected
        );
    }
}
