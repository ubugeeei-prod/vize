//! Frozen parser abort must complete semantics in the actual configured owner.
#![cfg(unix)]
use super::semantic_completion;
use crate::batch::{VirtualProject, executor::CorsaExecutor};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use vize_l0::{String, cstr};

const CASE: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/typechecker/native-semantic-completion/case.json"
);

#[test]
fn native_semantic_completion_keeps_frozen_full_vector_and_actual_terminal_owner() {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return;
    }
    let native = vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
        },
    )
    .expect("required real native configured factory");
    let temp = tempfile::tempdir().unwrap();
    let capture_root = std::env::var_os("VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| temp.path().to_path_buf());
    let root = capture_root.join("factory-escaped");
    fs::create_dir_all(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let case: Value = serde_json::from_str(CASE).unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker/native-semantic-completion");
    for entry in case["inputFiles"].as_array().unwrap() {
        let bytes = fs::read(fixture.join(entry["fixture"].as_str().unwrap())).unwrap();
        let mut digest = String::with_capacity(64);
        const HEX: &[u8] = b"0123456789abcdef";
        for b in Sha256::digest(&bytes) {
            digest.push(HEX[usize::from(b >> 4)] as char);
            digest.push(HEX[usize::from(b & 15)] as char)
        }
        assert_eq!(digest.as_str(), entry["sha256"].as_str().unwrap());
        assert_eq!(bytes.len(), entry["bytes"].as_u64().unwrap() as usize);
        let path = root.join(entry["file"].as_str().unwrap());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert_eq!(fs::read(path).unwrap(), bytes);
    }
    let mut project = VirtualProject::new(&root).unwrap();
    project.set_tsconfig_path(Some(root.join("tsconfig.json")));
    let files: Vec<_> = case["reportedFiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| root.join(v.as_str().unwrap()))
        .collect();
    project.register_paths(&files).unwrap();
    project.register_reachable_dependencies().unwrap();
    project.ensure_included_sources().unwrap();
    // Match the public runner's authored TS/JS diagnostic workload.
    project.set_diagnostic_paths(files.iter().map(PathBuf::as_path));
    let executor = CorsaExecutor::with_corsa_path(&root, Some(&native)).unwrap();
    semantic_completion::begin_for_test();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        executor.check_with_servers(&project, Some(1))
    }));
    let terminal = semantic_completion::finish();
    let result = result.unwrap().unwrap();
    let mut whole:Vec<_>=result.diagnostics.iter().map(|row|json!({"file":row.file.strip_prefix(&root).unwrap(),"line":row.line,"column":row.column,"code":row.code,"severity":row.severity,"message":row.message.as_str(),"blockType":row.block_type.map(|block|cstr!("{block:?}"))})).collect();
    whole.sort_by(|a, b| a["file"].as_str().cmp(&b["file"].as_str()));
    if let Some(dir) = std::env::var_os("VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR").map(PathBuf::from) {
        fs::write(dir.join("factory-escaped/result.json"),serde_json::to_vec_pretty(&json!({"nativeBinary":native,"diagnostics":whole,"success":result.success,"exitCode":result.exit_code})).unwrap()).unwrap();
    }
    {
        assert_eq!(terminal.as_array().unwrap().len(), 1);
        let receipt = &terminal[0];
        assert_eq!(receipt["completed"], true);
        assert_eq!(receipt["stderrComplete"], true);
        assert_eq!(receipt["exitCode"], 0);
        assert_eq!(receipt["timedOut"], false);
        assert_eq!(receipt["killRequested"], false);
        assert_eq!(receipt["reaped"], true);
        assert_eq!(receipt["stderrError"], Value::Null);
        assert_eq!(receipt["processError"], Value::Null);
        assert_eq!(receipt["stderr"], json!(b"".as_slice()));
    }
    assert_eq!(json!(whole), case["expectedDiagnostics"]);
    assert_eq!((result.success, result.exit_code), (false, 1));

    // This separate positive control checks only the five generated program
    // URIs. An ordinary import exposes the leaf global there; it does not claim
    // authored shared.ts is clean. The negative keeps all eight frozen inputs.
    let valid_root = root.parent().unwrap().join("factory-valid-import");
    fs::create_dir_all(&valid_root).unwrap();
    for entry in case["inputFiles"].as_array().unwrap() {
        let source = root.join(entry["file"].as_str().unwrap());
        let destination = valid_root.join(entry["file"].as_str().unwrap());
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::copy(source, destination).unwrap();
    }
    let original = fs::read_to_string(valid_root.join("Comp0.vue")).unwrap();
    assert_eq!(original.matches("i\\u006dport").count(), 1);
    fs::write(
        valid_root.join("Comp0.vue"),
        original.replace("i\\u006dport", "import"),
    )
    .unwrap();
    let mut valid = VirtualProject::new(&valid_root).unwrap();
    valid.set_tsconfig_path(Some(valid_root.join("tsconfig.json")));
    valid
        .register_paths(
            &case["reportedFiles"]
                .as_array()
                .unwrap()
                .iter()
                .map(|file| valid_root.join(file.as_str().unwrap()))
                .collect::<Vec<_>>(),
        )
        .unwrap();
    valid.register_reachable_dependencies().unwrap();
    valid.ensure_included_sources().unwrap();
    valid.materialize().unwrap();
    semantic_completion::begin_for_test();
    let valid_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        executor.check_with_project_session(&valid)
    }));
    let valid_terminal = semantic_completion::finish();
    let valid_result = valid_result.unwrap().unwrap();
    if let Some(dir) = std::env::var_os("VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR").map(PathBuf::from) {
        fs::write(dir.join("factory-valid-import/result.json"), serde_json::to_vec_pretty(&json!({
            "nativeBinary":native,"diagnostics":valid_result.diagnostics.iter().map(|row|cstr!("{row:?}")).collect::<Vec<_>>(),
            "success":valid_result.success,"exitCode":valid_result.exit_code,"terminal":valid_terminal,
        })).unwrap()).unwrap();
    }
    {
        let [receipt] = valid_terminal.as_array().unwrap().as_slice() else {
            panic!("one owned native process required")
        };
        assert_eq!(receipt["completed"], true);
        assert_eq!(receipt["stderrComplete"], true);
        assert_eq!(receipt["exitCode"], 0);
        assert_eq!(receipt["timedOut"], false);
        assert_eq!(receipt["killRequested"], false);
        assert_eq!(receipt["reaped"], true);
        assert_eq!(receipt["stderrError"], Value::Null);
        assert_eq!(receipt["processError"], Value::Null);
        assert_eq!(receipt["stderr"], json!(b"".as_slice()));
    }
    assert_eq!(valid_result.diagnostics.len(), 0);
    assert_eq!((valid_result.success, valid_result.exit_code), (true, 0));
}
