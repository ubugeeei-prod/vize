//! Same-owner native cold/warm controls use the unchanged original seven roots.

use super::*;
use vize_canon::{BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait};

fn expected_result(case: &Value, broken: bool) -> Value {
    let mut diagnostics = Vec::new();
    if broken {
        for (file, rows) in case["brokenDiagnostics"].as_object().unwrap() {
            for row in rows.as_array().unwrap() {
                diagnostics.push(json!({"file":file,"rendered":row}));
            }
        }
    }
    diagnostics.sort_by_key(Value::to_string);
    json!({"success":!broken,"exitCode":i32::from(broken),
        "files":case["reportedFiles"],"diagnostics":diagnostics})
}

fn result_view(
    checker: &BatchTypeChecker,
    root: &Path,
    result: &Result<vize_canon::batch::TypeCheckResult, vize_canon::batch::CorsaError>,
) -> Value {
    let Ok(result) = result else {
        return json!({"error":result.as_ref().err().unwrap().to_string()});
    };
    let mut files = checker
        .virtual_files()
        .into_iter()
        .map(|file| {
            file.original_path
                .strip_prefix(root)
                .unwrap_or(&file.original_path)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<Vec<_>>();
    files.sort();
    let mut diagnostics = result.diagnostics.iter().map(|row| {
        let severity = match row.severity {1=>cstr!("error"),2=>cstr!("warning"),3=>cstr!("info"),4=>cstr!("hint"),other=>cstr!("unknown-severity-{other}")};
        let code = row.code.map(|code| cstr!(" [TS{code}]")).unwrap_or_default();
        json!({"file":row.file.strip_prefix(root).unwrap_or(&row.file).to_string_lossy().replace('\\', "/"),
            "rendered":cstr!("{severity}:{}:{}{code} {}",u64::from(row.line)+1,u64::from(row.column)+1,row.message).as_str()})
    }).collect::<Vec<_>>();
    diagnostics.sort_by_key(Value::to_string);
    json!({"success":result.success,"exitCode":result.exit_code,"files":files,"diagnostics":diagnostics})
}

#[test]
fn original_literal_roots_reuse_one_native_owner_and_clean_its_storage() {
    let Some(native) = corsa_requirement::required_or_skip(None::<PathBuf>) else {
        return;
    };
    assert_eq!(
        packet(Command::new(&native).arg("--version")),
        json!({"exitCode":0,"stdout":"Version 7.0.2\n","stderr":""})
    );
    let corpus: Value = serde_json::from_str(INPUT).unwrap();
    let case = &corpus["cases"]["literal-dependency"];
    let root = literal_package::project("warm-owner", &corpus);
    let paths = case["programFiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| root.join(path.as_str().unwrap()))
        .collect::<Vec<_>>();
    // Declared before the checker: unwind drops the checker before recording cleanup.
    let mut cleanup_receipt = CleanupReceipt {
        storage: None,
        namespace: None,
    };
    let scan_started = std::time::Instant::now();
    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        &root,
        BatchTypeCheckerOptions {
            tsconfig_path: Some(root.join("tsconfig.json")),
            ..Default::default()
        },
        Some(&native),
    )
    .unwrap();
    checker.scan_paths(&paths).unwrap();
    let scan_ns = scan_started.elapsed().as_nanos();
    let namespace = checker
        .virtual_files()
        .into_iter()
        .find(|file| file.original_path == root.join("src/entry.ts"))
        .unwrap()
        .virtual_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let storage = namespace.parent().unwrap().parent().unwrap().to_path_buf();
    cleanup_receipt.storage = Some(storage.clone());
    cleanup_receipt.namespace = Some(namespace.clone());
    assert_ne!(namespace, vize_canon::batch::project_virtual_root(&root));
    assert!(!namespace.starts_with(&root));
    let virtual_paths = checker
        .virtual_files()
        .into_iter()
        .map(|file| file.virtual_path.clone())
        .collect::<Vec<_>>();
    let mut observations = Vec::new();
    let mut assertions = Vec::new();
    for phase in ["clean", "broken", "repair"] {
        write(
            &root,
            if phase == "broken" {
                &corpus["brokenFiles"]
            } else {
                &corpus["commonFiles"]
            },
        );
        literal_package::stock("warm-owner", phase, &root, &native, case);
        for repeat in 0..2 {
            let package_before = literal_package::receipt(&root.join("node_modules"));
            let started = std::time::Instant::now();
            let result = checker.check_incremental(&paths);
            let elapsed_ns = started.elapsed().as_nanos();
            let whole = result_view(&checker, &root, &result);
            let metrics = checker.incremental_metrics();
            let package_after = literal_package::receipt(&root.join("node_modules"));
            observations.push(json!({"phase":phase,"repeat":repeat,"elapsedNs":elapsed_ns,
                "wholeResult":whole,"allResult":cstr!("{result:?}").as_str(),"packageBefore":package_before,"packageAfter":package_after,
                "allMetrics":cstr!("{metrics:?}").as_str()}));
            if let Some(capture) = std::env::var_os("VIZE_TSCONFIG_TYPES_CAPTURE") {
                let dir = PathBuf::from(capture).join("ignore-literal/warm-owner");
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(dir.join("runtime.json"), serde_json::to_vec_pretty(&json!({
                    "wholeInput":corpus,"sourceSha":std::env::var("SOURCE_SHA").ok(),
                    "nativeBinary":native,"projectRoot":root,"namespace":namespace,"storage":storage,
                    "scanNs":scan_ns,"observations":observations})).unwrap()).unwrap();
            }
            assertions.push((
                phase,
                repeat,
                whole,
                metrics,
                package_before,
                package_after,
                checker
                    .virtual_files()
                    .into_iter()
                    .map(|file| file.virtual_path.clone())
                    .collect::<Vec<_>>(),
            ));
        }
    }
    let final_metrics = checker.incremental_metrics();
    let existed_before_drop = storage.is_dir();
    drop(checker);
    cleanup_receipt.write();
    assert!(existed_before_drop);
    assert!(!storage.exists());
    for (phase, repeat, whole, metrics, package_before, package_after, observed_paths) in assertions
    {
        assert_eq!(
            package_after, package_before,
            "raw native owner conservation"
        );
        assert_eq!(
            whole,
            expected_result(case, phase == "broken"),
            "{phase}/{repeat}"
        );
        assert_eq!(observed_paths, virtual_paths);
        assert_eq!(metrics.session_to_cli_fallbacks, 0);
        assert_eq!(metrics.session_starts, 1);
        if phase == "clean" && repeat == 0 {
            assert!(metrics.last_session_started);
        } else {
            assert!(metrics.last_session_reused);
            assert_eq!(metrics.last_tree_entries_scanned, 0);
            assert!(!metrics.last_full_rebuild);
        }
    }
    assert_eq!(final_metrics.checks, 6);
    assert_eq!(final_metrics.session_reuses, 5);
    println!(
        "complete configured ignore warm-owner: 6 actual native API calls, one start/five reuses, exact original roots/diagnostics and cleanup"
    );
    std::fs::remove_dir_all(root).unwrap();
}

struct CleanupReceipt {
    storage: Option<PathBuf>,
    namespace: Option<PathBuf>,
}

impl CleanupReceipt {
    fn write(&self) {
        let Some(capture) = std::env::var_os("VIZE_TSCONFIG_TYPES_CAPTURE") else {
            return;
        };
        let dir = PathBuf::from(capture).join("ignore-literal/warm-owner");
        let cleanup = json!({"sourceSha":std::env::var("SOURCE_SHA").ok(),
            "storage":self.storage,"namespace":self.namespace,
            "storageExistsAfterDrop":self.storage.as_ref().map(|path|path.exists()),
            "namespaceExistsAfterDrop":self.namespace.as_ref().map(|path|path.exists())});
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("cleanup.json"),
            serde_json::to_vec_pretty(&cleanup).unwrap(),
        )
        .unwrap();
    }
}

impl Drop for CleanupReceipt {
    fn drop(&mut self) {
        // Never replace an existing assertion panic with a capture write panic.
        let _ = std::panic::catch_unwind(|| self.write());
    }
}
