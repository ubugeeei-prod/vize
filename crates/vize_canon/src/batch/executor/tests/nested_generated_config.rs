//! Both Batch session starts must preserve the whole governing nested config.

use std::{fs, path::Path};

use crate::batch::{BatchTypeChecker, BatchTypeCheckerOptions, TypeCheckResult, TypeChecker};

const APP: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/typechecker/nested-batch-config/App.vue.txt"
);
const VALUE: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/typechecker/nested-batch-config/value.ts.txt"
);
const CONFIG: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/typechecker/nested-batch-config/playground.tsconfig.json.txt"
);
const CASE: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/typechecker/nested-batch-config/case.json"
);

#[test]
fn both_batch_session_paths_use_the_nested_config_and_keep_exact_warm_deltas() {
    let native = vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
        },
    )
    .expect("the nested native corpus requires the actual backend");
    verify_originals();
    for (case_index, root_config) in [
        None,
        Some(r#"{"compilerOptions":{"strict":false},"include":["src/**/*"]}"#),
    ]
    .into_iter()
    .enumerate()
    {
        let root = tempfile::tempdir().unwrap();
        for (path, source) in [
            ("src/App.vue", APP),
            ("src/value.ts", VALUE),
            ("playground/tsconfig.json", CONFIG),
        ] {
            let target = root.path().join(path);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(&target, source).unwrap();
            assert_eq!(fs::read(&target).unwrap(), source.as_bytes());
        }
        if let Some(config) = root_config {
            fs::write(root.path().join("tsconfig.json"), config).unwrap();
        }
        let options = BatchTypeCheckerOptions {
            tsconfig_path: Some(root.path().join("playground/tsconfig.json")),
            ..Default::default()
        };
        let mut checker =
            BatchTypeChecker::with_options_and_corsa_path(root.path(), options, Some(&native))
                .unwrap();
        checker.scan_project().unwrap();
        let app = root.path().join("src/App.vue");
        // Exercise the nonincremental session path itself, rather than its CLI-first wrapper.
        let mut project = crate::batch::VirtualProject::new(root.path()).unwrap();
        project.set_tsconfig_path(Some(root.path().join("playground/tsconfig.json")));
        project
            .register_paths(&[app.clone(), root.path().join("src/value.ts")])
            .unwrap();
        project.register_reachable_dependencies().unwrap();
        project.ensure_included_sources().unwrap();
        project.materialize().unwrap();
        let generated = project.generated_tsconfig_path();
        assert_eq!(
            generated,
            project.virtual_root().join("playground/tsconfig.json")
        );
        assert!(!project.virtual_root().join("tsconfig.json").exists());
        native_oracle(&native, &project, case_index);
        let executor = super::CorsaExecutor::with_corsa_path(root.path(), Some(&native)).unwrap();
        let ordinary = executor.check_with_project_session(&project).unwrap();
        assert_original_error(root.path(), &ordinary);
        let cold = checker
            .check_incremental(std::slice::from_ref(&app))
            .unwrap();
        assert_original_error(root.path(), &cold);
        assert_eq!(
            whole_vector(root.path(), &ordinary),
            whole_vector(root.path(), &cold)
        );
        let cold_metrics = checker.incremental_metrics();
        assert_eq!(
            (
                cold_metrics.checks,
                cold_metrics.session_starts,
                cold_metrics.session_to_cli_fallbacks
            ),
            (1, 1, 0)
        );

        let repaired = APP.replace("required: number =", "required: number | undefined =");
        fs::write(&app, &repaired).unwrap();
        let warm = checker
            .check_incremental(std::slice::from_ref(&app))
            .unwrap();
        assert!(
            warm.success && warm.exit_code == 0 && warm.diagnostics.is_empty(),
            "{warm:#?}"
        );
        assert_delta(checker.incremental_metrics(), 2, 1);
        fs::write(&app, APP).unwrap();
        let broken = checker
            .check_incremental(std::slice::from_ref(&app))
            .unwrap();
        assert_eq!(
            whole_vector(root.path(), &cold),
            whole_vector(root.path(), &broken)
        );
        assert_delta(checker.incremental_metrics(), 3, 2);
        save_vectors(case_index, root.path(), [&ordinary, &cold, &warm, &broken]);
        assert_eq!(
            fs::read(root.path().join("playground/tsconfig.json")).unwrap(),
            CONFIG.as_bytes()
        );
        assert_eq!(
            fs::read(root.path().join("src/value.ts")).unwrap(),
            VALUE.as_bytes()
        );
    }
}

fn verify_originals() {
    use sha2::{Digest, Sha256};
    let case: serde_json::Value = serde_json::from_str(CASE).unwrap();
    for (name, bytes) in [
        ("App.vue.txt", APP),
        ("value.ts.txt", VALUE),
        ("playground.tsconfig.json.txt", CONFIG),
    ] {
        assert_eq!(case["originals"][name]["bytes"], bytes.len());
        assert_eq!(
            case["originals"][name]["sha256"],
            vize_l0::cstr!("{:x}", Sha256::digest(bytes.as_bytes())).as_str()
        );
    }
}

fn whole_vector(root: &Path, result: &TypeCheckResult) -> serde_json::Value {
    serde_json::json!({ "success": result.success, "exitCode": result.exit_code,
        "diagnostics": result.diagnostics.iter().map(|d| serde_json::json!({
            "file": d.file.strip_prefix(root).unwrap(), "line": d.line,
            "column": d.column, "message": d.message, "code": d.code,
            "severity": d.severity, "blockType": d.block_type.map(|b| vize_l0::cstr!("{b:?}")),
        })).collect::<Vec<_>>() })
}

fn assert_original_error(root: &Path, result: &TypeCheckResult) {
    assert!(!result.success && result.exit_code == 1, "{result:#?}");
    assert_eq!(result.diagnostics.len(), 1, "{result:#?}");
    let d = &result.diagnostics[0];
    let expected: serde_json::Value = serde_json::from_str(CASE).unwrap();
    assert_eq!(
        d.file,
        root.join(expected["expected"]["file"].as_str().unwrap())
    );
    assert_eq!(
        (d.code, d.line, d.column, d.severity),
        (Some(2322), 2, 6, 1)
    );
    assert_eq!(d.block_type, Some(crate::batch::SfcBlockType::ScriptSetup));
    assert_eq!(
        d.message.as_str(),
        expected["expected"]["message"].as_str().unwrap()
    );
}

fn assert_delta(m: crate::batch::IncrementalCheckMetrics, checks: usize, reuses: usize) {
    assert_eq!(
        (
            m.checks,
            m.session_starts,
            m.session_reuses,
            m.session_refreshes,
            m.session_to_cli_fallbacks
        ),
        (checks, 1, reuses, reuses, 0)
    );
    assert!(m.last_session_reused && m.last_session_refreshed && !m.last_full_rebuild);
    assert_eq!(
        (
            m.last_changed_files,
            m.last_created_files,
            m.last_deleted_files,
            m.last_tree_entries_scanned
        ),
        (1, 0, 0, 0)
    );
}

fn save_vectors(case_index: usize, root: &Path, results: [&TypeCheckResult; 4]) {
    if let Some(dir) = std::env::var_os("VIZE_NESTED_BATCH_CAPTURE_DIR") {
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            Path::new(&dir).join(vize_l0::cstr!("case-{case_index}-mapped.json").as_str()),
            serde_json::to_vec_pretty(&results.map(|result| whole_vector(root, result))).unwrap(),
        )
        .unwrap();
    }
}

#[expect(
    clippy::string_slice,
    reason = "verified ASCII declaration prefix in complete generated source"
)]
fn native_oracle(native: &Path, project: &crate::batch::VirtualProject, case_index: usize) {
    let config = project.generated_tsconfig_path();
    let original = fs::read(&config).unwrap();
    let mut weak: serde_json::Value = serde_json::from_slice(&original).unwrap();
    weak["compilerOptions"]["strict"] = serde_json::json!(false);
    let weak_path = config.with_file_name("weak.native.json");
    let weak_bytes = serde_json::to_vec_pretty(&weak).unwrap();
    fs::write(&weak_path, &weak_bytes).unwrap();
    let observations = [&config, &weak_path].map(|path| {
        let output = std::process::Command::new(native)
            .current_dir(project.virtual_root())
            .args(["--noEmit", "--pretty", "false", "--project"])
            .arg(path).output().unwrap();
        serde_json::json!({ "native": native, "args": ["--noEmit", "--pretty", "false", "--project", path.to_str().unwrap()],
            "cwd": project.virtual_root(), "status": output.status.code(),
            "stdout": std::str::from_utf8(&output.stdout).unwrap(),
            "stderr": std::str::from_utf8(&output.stderr).unwrap(),
            "configBytes": fs::read_to_string(path).unwrap() })
    });
    fs::remove_file(&weak_path).unwrap();
    assert_eq!(fs::read(&config).unwrap(), original);
    if let Some(dir) = std::env::var_os("VIZE_NESTED_BATCH_CAPTURE_DIR") {
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            Path::new(&dir).join(vize_l0::cstr!("case-{case_index}-native-cli.json").as_str()),
            serde_json::to_vec_pretty(&observations).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(observations[0]["status"], 1, "{observations:#?}");
    assert_eq!(observations[0]["stderr"], "", "{observations:#?}");
    let app = project
        .virtual_files_sorted()
        .into_iter()
        .find(|file| file.original_path.ends_with("src/App.vue"))
        .unwrap();
    let marker = "const required: number = maybeNumber";
    let declaration = app.content.find(marker).unwrap();
    let before = &app.content[..declaration + "const ".len()];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before.rsplit('\n').next().unwrap().chars().count() + 1;
    let descriptor: serde_json::Value = serde_json::from_str(CASE).unwrap();
    let expected_stdout = vize_l0::cstr!(
        "{}({line},{column}): error TS2322: {}\n",
        app.virtual_path
            .strip_prefix(project.virtual_root())
            .unwrap()
            .display(),
        descriptor["expected"]["message"].as_str().unwrap()
    );
    assert_eq!(
        observations[0]["stdout"],
        expected_stdout.as_str(),
        "{observations:#?}"
    );
    assert_eq!(observations[1]["status"], 0, "{observations:#?}");
    assert_eq!(observations[1]["stdout"], "", "{observations:#?}");
    assert_eq!(observations[1]["stderr"], "", "{observations:#?}");
}
