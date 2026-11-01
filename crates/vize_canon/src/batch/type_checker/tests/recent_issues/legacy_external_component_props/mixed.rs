//! Complete controlled observations of mixed legacy scalar/callback props.
//! Synthetic declarations retain all findings before the scalar-only assertion.
use crate::batch::TypeChecker;
use std::path::Path;

use serde_json::{Value, json};

use super::{
    BatchTypeChecker, BatchTypeCheckerOptions, VirtualTsOptions, create_project_case,
    resolve_test_tsgo_binary,
};
use vize_carton::cstr;

const INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/legacy-mixed-scalar-props/input.json"
));
const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/legacy-external-component-props/loose-controls.json"
));

#[test]
fn mixed_known_scalar_props_preserve_callbacks_and_all_existing_findings() {
    assert!(
        std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none()
            || std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_none(),
        "required mixed native control cannot be disabled"
    );
    let Some(native) = resolve_test_tsgo_binary() else {
        return;
    };
    let input: Value = serde_json::from_str(INPUT).expect("pinned mixed input");
    assert_eq!(
        input["id"],
        "typechecker/regression/legacy-mixed-scalar-props"
    );
    let original: Value = serde_json::from_str(ORIGINAL).expect("delivered original input");
    let original_mixed = original["files"]
        .as_array()
        .expect("original files")
        .iter()
        .find(|file| file["path"] == "src/MixedCallbacks.vue")
        .expect("original mixed case");
    assert_eq!(input["files"][0]["source"], original_mixed["source"]);
    let mut observations = Vec::new();
    for legacy in [false, true] {
        for phase in ["bad", "repair", "restore"] {
            observations.push(observe_with_source(&input, legacy, phase));
        }
    }
    let capture = json!({
        "schema":"vize.legacy-mixed-scalar-api-observation.v1",
        "scope":"Complete six batch API observations before any semantic assertions. Synthetic constructor types, no installed framework/application claim. Callback-body/unknown/public-props/unregistered/shadow findings remain whole.",
        "requestedNativePath":native, "input":input, "observations":observations,
    });
    let whole = serde_json::to_string_pretty(&capture).expect("whole mixed API capture");
    if let Some(root) = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR") {
        let directory = std::path::PathBuf::from(root).join("legacy-external-props");
        std::fs::create_dir_all(&directory).expect("capture directory");
        std::fs::write(directory.join("mixed-scalar-observations.json"), &whole)
            .expect("retain every full result before scalar assertion");
    }
    println!("{whole}");
    // Pin every actual producer diagnostic, including preexisting callback,
    // public-props, shadow and unregistered findings. No record is filtered.
    for observation in capture["observations"]
        .as_array()
        .expect("six observations")
    {
        assert_eq!(observation["kind"], "result", "{observation:#}");
        assert_eq!(observation["exitCode"], 1);
        assert_eq!(observation["success"], false);
        let root = Path::new(observation["projectRoot"].as_str().expect("root"));
        let legacy = observation["legacy"].as_bool().expect("dialect");
        let phase = observation["phase"].as_str().expect("phase");
        let expected_source = match (legacy, phase) {
            (false, "bad") => include_str!("mixed/expected/modern-bad.json"),
            (false, "repair") => include_str!("mixed/expected/modern-repair.json"),
            (false, "restore") => include_str!("mixed/expected/modern-restore.json"),
            (true, "bad") => include_str!("mixed/expected/legacy-bad.json"),
            (true, "repair") => include_str!("mixed/expected/legacy-repair.json"),
            (true, "restore") => include_str!("mixed/expected/legacy-restore.json"),
            _ => panic!("unknown observation identity"),
        };
        let mut expected: Value =
            serde_json::from_str(expected_source).expect("whole observed oracle");
        for diagnostic in expected.as_array_mut().expect("complete vector") {
            diagnostic["file"] =
                json!(root.join(diagnostic["file"].as_str().expect("logical source")));
        }
        assert_eq!(
            observation["diagnostics"], expected,
            "complete exact observed vector: {observation:#}"
        );
        for file in observation["virtualFiles"]
            .as_array()
            .expect("materialized files")
        {
            assert_eq!(file["disk"]["kind"], "text");
            assert_eq!(
                file["content"], file["disk"]["content"],
                "same-checker physical producer bytes"
            );
        }
    }
}

fn observe_with_source(input: &Value, legacy: bool, phase: &str) -> Value {
    let files = input["files"]
        .as_array()
        .expect("authored files")
        .iter()
        .map(|file| {
            let path = file["path"].as_str().expect("authored path");
            let source = if path == "src/External.vue" && phase == "repair" {
                input["repairedExternal"].as_str().expect("authored repair")
            } else {
                file["source"].as_str().expect("authored source")
            };
            (path, source)
        })
        .collect::<Vec<_>>();
    // Record the exact input passed to the ordinary producer; no on-disk
    // source is changed after the scan or reused between these observations.
    let sources = files
        .iter()
        .map(|(path, source)| json!({"path":path,"source":source}))
        .collect::<Vec<_>>();
    let root = create_project_case(
        cstr!("legacy-external-props-{legacy}-{phase}").as_str(),
        &files,
    );
    let options = BatchTypeCheckerOptions {
        virtual_ts_options: VirtualTsOptions {
            auto_import_stubs: input["externalDeclarations"]
                .as_array()
                .expect("declarations")
                .iter()
                .map(|value| value.as_str().expect("declaration").into())
                .collect(),
            external_template_bindings: input["externalBindings"]
                .as_array()
                .expect("binding names")
                .iter()
                .map(|value| value.as_str().expect("binding name").into())
                .collect(),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut checker = BatchTypeChecker::with_options(&root, options).expect("batch construction");
    if legacy {
        checker.enable_legacy_vue2();
    }
    let mut result = match checker.scan_project().and_then(|_| checker.check_project()) {
        Ok(result) => json!({
            "kind":"result", "legacy":legacy, "phase":phase, "projectRoot":root,
            "sources":sources, "exitCode":result.exit_code, "success":result.success,
            "diagnostics":result.diagnostics.iter().map(|diagnostic| json!({
                "file":diagnostic.file, "line":diagnostic.line, "column":diagnostic.column,
                "message":diagnostic.message, "code":diagnostic.code, "severity":diagnostic.severity,
                "blockType":format!("{:?}",diagnostic.block_type),
            })).collect::<Vec<_>>(),
        }),
        Err(error) => json!({"kind":"error", "legacy":legacy, "phase":phase,
            "projectRoot":root, "sources":sources, "error":format!("{error:?}")}),
    };
    // Capture the same checker's materialized producer bytes while its cache
    // lease is still held. No second generator or parser supplies this oracle.
    let virtual_files = checker
        .virtual_files()
        .into_iter()
        .map(|file| {
            let disk = match std::fs::read_to_string(&file.virtual_path) {
                Ok(content) => json!({"kind":"text", "content":content}),
                Err(error) => json!({"kind":"error", "message":format!("{error:?}")}),
            };
            json!({"originalPath":file.original_path, "virtualPath":file.virtual_path,
            "content":file.content.as_str(), "disk":disk})
        })
        .collect::<Vec<_>>();
    result["virtualFiles"] = json!(virtual_files);
    result["sharedHelpers"] = json!(checker.shared_helpers_preamble());
    drop(checker);
    std::fs::remove_dir_all(&root)
        .expect("remove source-only temporary project after native completion");
    result
}
