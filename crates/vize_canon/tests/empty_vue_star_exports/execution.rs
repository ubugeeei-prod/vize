//! Persist complete pending receipts before each genuine operation or assertion.

use super::{
    membership, qualification,
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
    let listing = support::cli(runtime, &config, &["--listFilesOnly", "--pretty", "false"]);
    packet["wholeNativeLoadedFileListing"] = listing.clone();
    support::capture(id, &packet);
    let (listed_paths, partition) = partition_listing(&packet, &listing);
    packet["wholeNativeLoadedFileListingPartition"] = partition;
    support::capture(id, &packet);
    let loaded = listed_paths
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
    if case["id"] == "malformed-refusal" {
        packet["completeOrderedMalformedMembership"] =
            membership::validate(&packet, &packet["wholeNativeLoadedFiles"]);
    }
    packet["phase"] = json!("SUPPORT_CUSTODY_VALIDATED_PENDING_SOURCE_AND_VECTOR_ASSERTIONS");
    support::capture(id, &packet);
    Observation {
        packet,
        generated: Value::Object(generated),
        rows,
    }
}

/// This one observed malformed case retains its native syntax failure verbatim.
/// It never recognizes arbitrary diagnostics or grants semantic-green credit.
fn partition_listing<'a>(packet: &Value, listing: &'a Value) -> (&'a str, Value) {
    let stdout = listing["wholeStdoutUtf8"].as_str().unwrap();
    if packet["case"]["id"] != "malformed-refusal" {
        assert_eq!(
            listing["exitCode"], 0,
            "native loaded-file custody is required"
        );
        return (stdout, json!({"kind":"ordinary-path-only-listing"}));
    }
    const SYNTAX: &str = "A.vue.ts(30,18): error TS1109: Expression expected.\n";
    let native = &packet["wholeSameConfigNativeCli"];
    assert_eq!(
        packet["case"]["nativeContract"],
        "projection-refusal-observation"
    );
    assert_eq!(
        listing["arguments"],
        json!(["--listFilesOnly", "--pretty", "false"])
    );
    assert_eq!(
        native["arguments"],
        json!(["--checkers", "1", "--pretty", "false"])
    );
    for identity in ["nativePath", "nativeSHA256", "project", "cwd"] {
        assert_eq!(
            listing[identity].as_str().unwrap(),
            native[identity].as_str().unwrap(),
            "the listing must belong to this exact native/config identity"
        );
    }
    assert_eq!(
        native["nativePath"],
        packet["officialNativeCustody"]["executable"]
    );
    assert_eq!(
        native["nativeSHA256"],
        packet["officialNativeCustody"]["wholeExecutableSHA256"]
    );
    assert_eq!(
        packet["officialNativeCustody"]["phase"],
        "OFFICIAL_NATIVE_RUNTIME_VALIDATED"
    );
    assert_eq!(native["project"], packet["generatedConfig"]["path"]);
    assert_eq!(listing["exitCode"], 1);
    assert_eq!(native["exitCode"], 1);
    for response in [native, listing] {
        assert_eq!(response["wholeStderrBytes"], json!([]));
        assert_eq!(response["stderrSha256"], support::hash(&[]));
        let text = response["wholeStdoutUtf8"].as_str().unwrap();
        assert_eq!(response["wholeStdoutBytes"], json!(text.as_bytes()));
        assert_eq!(response["stdoutSha256"], support::hash(text.as_bytes()));
    }
    assert_eq!(
        native["wholeStdoutUtf8"], SYNTAX,
        "only the exact whole observed syntax response is eligible"
    );
    let paths = stdout
        .strip_prefix(SYNTAX)
        .expect("the exact whole syntax response must occur once at the beginning");
    assert_eq!(
        paths,
        paths.lines().collect::<Vec<_>>().join("\n") + "\n",
        "the complete filepath listing must retain every line and its final newline"
    );
    for line in paths.lines() {
        let path = std::path::Path::new(line);
        assert!(
            path.is_absolute(),
            "no second or foreign diagnostic may become a filepath"
        );
        assert!(
            matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("ts" | "mts")
            ),
            "only the actual malformed case's native source-file path shape is eligible"
        );
    }
    (
        paths,
        json!({"kind":"single-malformed-case-syntax-prefix", "wholeSyntaxUtf8":SYNTAX,
        "wholeSyntaxBytes":SYNTAX.as_bytes(),"syntaxSHA256":support::hash(SYNTAX.as_bytes()),
        "wholeSameConfigNativeSyntaxResponse":native,
        "authority":"exact same native/config CLI syntax failure retained; no semantic-green credit"}),
    )
}
