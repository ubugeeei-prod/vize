//! Persist complete pending receipts before each genuine operation or assertion.

use super::{
    qualification,
    support::{self, Runtime},
};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};
use vize_canon::{BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait};

pub(super) struct Observation {
    pub(super) packet: Value,
    pub(super) generated: Value,
    pub(super) rows: Option<Value>,
}

pub(super) fn check(case: &Value, runtime: &Runtime) -> Observation {
    let id = case["id"].as_str().unwrap();
    let inputs = case["inputs"].as_array().unwrap();
    let mut packet = json!({"case":case,"phase":"PENDING_FIXTURE_AND_PROVIDER_CUSTODY",
        "officialNativeCustody":runtime.native_custody,
        "wholeFixtureInputBodies":inputs.iter().map(support::blob).collect::<Vec<_>>(),
        "nativeAcceptance":"pending; no native acceptance credit",
        "scope":"ordinary configured batch and separate native CLI; no LSP/API/PID authority credit"});
    support::capture(id, &packet);
    let providers = qualification::provider_reads(runtime);
    packet["genuineQualifiedProviders"] = providers.clone();
    packet["phase"] = json!("PENDING_PROVIDER_VALIDATION");
    support::capture(id, &packet);
    qualification::providers(&providers);
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    fs::create_dir(root.join("node_modules")).unwrap();
    support::link(&runtime.vue, &root.join("node_modules/vue"));
    support::link(&runtime.vite, &root.join("node_modules/vite"));
    for input in inputs {
        let path = root.join(support::relative(input["file"].as_str().unwrap()));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, support::body(input)).unwrap();
    }
    let authored_config = root.join("tsconfig.json");
    packet["authoredInputs"] = json!(
        inputs
            .iter()
            .map(|input| {
                support::observed_custody(
                    &root.join(support::relative(input["file"].as_str().unwrap())),
                )
            })
            .collect::<Vec<_>>()
    );
    packet["authoredConfigPath"] = json!(authored_config);
    packet["phase"] = json!("PENDING_NORMAL_BATCH_CONSTRUCTION_SCAN_AND_CHECK");
    support::capture(id, &packet);
    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        &root,
        BatchTypeCheckerOptions {
            tsconfig_path: Some(authored_config),
            ..Default::default()
        },
        Some(&runtime.native),
    )
    .unwrap();
    checker.set_server_count(Some(1));
    checker.set_checker_count(Some(1));
    checker.scan_project().unwrap();
    let result = checker.check_project();
    packet["wholeBatchResult"] = json!(format!("{result:#?}"));
    packet["phase"] = json!("BATCH_RETURNED_PENDING_CUSTODY_AND_ASSERTIONS");
    support::capture(id, &packet);
    let mirror = vize_canon::project_virtual_root(&root);
    let config = mirror.join("tsconfig.json");
    packet["generatedConfig"] = support::observed_custody(&config);
    let mut generated = serde_json::Map::new();
    let program = checker.virtual_files().iter().map(|file| {
        let relative = file.original_path.strip_prefix(&root).unwrap_or(&file.original_path).to_string_lossy().replace('\\', "/");
        generated.insert(relative, json!(file.content.as_str()));
        json!({"originalPath":file.original_path,"virtualPath":file.virtual_path,
            "wholeAuthored":support::observed_custody(&file.original_path),
            "wholeCode":file.content.as_str(),"generatedSHA256":support::hash(file.content.as_bytes()),
            "actualMaterializedRead":support::observed_custody(&file.virtual_path),
            "wholeSourceMap":format!("{:#?}",file.source_map)})
    }).collect::<Vec<_>>();
    packet["wholeGeneratedProgram"] = json!(program);
    let rows = result.as_ref().ok().map(|result| json!(result.diagnostics.iter().map(|d| {
        json!({"file":d.file.strip_prefix(&root).unwrap_or(&d.file).to_string_lossy().replace('\\', "/"),
            "line":d.line+1,"column":d.column+1,"severity":d.severity,"code":d.code,"message":d.message.as_str()})
    }).collect::<Vec<_>>()));
    packet["wholeMappedDiagnosticVector"] = json!(rows);
    packet["success"] = json!(result.as_ref().ok().map(|result| result.success));
    packet["exitCode"] = json!(result.as_ref().ok().map(|result| result.exit_code));
    packet["phase"] = json!("BATCH_AND_GENERATED_CUSTODY_COMPLETE_PENDING_ASSERTIONS");
    support::capture(id, &packet);
    let native = support::cli(runtime, &config, &["--checkers", "1", "--pretty", "false"]);
    packet["wholeSameConfigNativeCli"] = native;
    packet["phase"] = json!("RAW_NATIVE_RETURNED_PENDING_ASSERTIONS");
    support::capture(id, &packet);
    let show = support::cli(runtime, &config, &["--showConfig"]);
    packet["wholeSameConfigNativeShowConfig"] = show.clone();
    support::capture(id, &packet);
    assert_eq!(
        show["exitCode"], 0,
        "native effective config must be observable"
    );
    let effective: Value = serde_json::from_str(show["wholeStdoutUtf8"].as_str().unwrap()).unwrap();
    packet["nativeEffectiveConfig"] = effective;
    support::capture(id, &packet);
    let listing = support::cli(runtime, &config, &["--listFilesOnly"]);
    packet["wholeNativeLoadedFileListing"] = listing.clone();
    support::capture(id, &packet);
    assert_eq!(
        listing["exitCode"], 0,
        "native loaded-file custody is required"
    );
    let loaded = listing["wholeStdoutUtf8"]
        .as_str()
        .unwrap()
        .lines()
        .map(|path| {
            let path = PathBuf::from(path);
            support::observed_custody(&if path.is_absolute() {
                path
            } else {
                mirror.join(path)
            })
        })
        .collect::<Vec<_>>();
    packet["wholeNativeLoadedFiles"] = json!(loaded);
    packet["phase"] = json!("WHOLE_SUPPORT_PACKET_COMPLETE_PENDING_ASSERTIONS");
    support::capture(id, &packet);
    // Every actual output/read above survives a genuine RED below.
    for file in packet["wholeGeneratedProgram"].as_array().unwrap() {
        assert_eq!(
            file["wholeCode"], file["actualMaterializedRead"]["wholeUtf8"],
            "native must consume the actual current producer bytes"
        );
    }
    for file in packet["wholeNativeLoadedFiles"].as_array().unwrap() {
        assert!(file.get("observedReadError").is_none(), "{file}");
        assert!(
            file["wholeUtf8"].is_string(),
            "complete native declaration bytes must be readable: {file}"
        );
    }
    qualification::configuration(case, &packet, &mirror);
    packet["actualNativeLoadedProviderRoles"] =
        qualification::loaded_roles(&providers, &packet["wholeNativeLoadedFiles"]);
    packet["phase"] = json!("SUPPORT_CUSTODY_VALIDATED_PENDING_SOURCE_AND_VECTOR_ASSERTIONS");
    support::capture(id, &packet);
    Observation {
        packet,
        generated: Value::Object(generated),
        rows,
    }
}
