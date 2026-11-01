//! Synthetic typed external props must survive the selected legacy dialect.
//!
//! These declarations isolate the checker boundary. They are not installed
//! Vue 2/Vuetify types or a reconstruction of the reported application.

use super::super::{create_project_case, resolve_test_tsgo_binary};
use crate::batch::{BatchTypeChecker, BatchTypeCheckerOptions, TypeChecker};
use crate::virtual_ts::VirtualTsOptions;
use serde_json::{Value, json};
use std::path::Path;
use vize_carton::cstr;

const INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/legacy-external-component-props/input.json"
));

#[test]
fn legacy_external_component_props_preserve_typed_values_and_unknown_fallbacks() {
    assert!(
        std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none()
            || std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_none(),
        "required native control cannot be disabled"
    );
    let Some(native) = resolve_test_tsgo_binary() else {
        return;
    };
    let input: Value = serde_json::from_str(INPUT).expect("pinned physical input JSON");
    assert_eq!(
        input["id"],
        "typechecker/regression/legacy-external-component-props"
    );
    let mut observations = Vec::new();
    for legacy in [false, true] {
        for phase in ["bad", "repair", "restore"] {
            observations.push(observe(&input, legacy, phase));
        }
    }
    let capture = json!({
        "schema": "vize.legacy-external-props-api-observation.v1",
        "scope": "Complete batch API results, not underlying native CLI stdout or an installed Vuetify application",
        "requestedNativePath": native,
        "input": input,
        "observations": observations,
    });
    let whole = serde_json::to_string_pretty(&capture).expect("complete API capture");
    if let Some(root) = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR") {
        let directory = std::path::PathBuf::from(root).join("legacy-external-props");
        std::fs::create_dir_all(&directory).expect("capture directory");
        std::fs::write(directory.join("api-observations.json"), &whole)
            .expect("retain every result before semantic assertions");
    }
    println!("{whole}");
    for observation in capture["observations"]
        .as_array()
        .expect("all six observations")
    {
        assert_eq!(observation["kind"], "result", "{observation:#}");
        let phase = observation["phase"].as_str().expect("phase");
        let diagnostics = observation["diagnostics"]
            .as_array()
            .expect("full diagnostics");
        let root = Path::new(observation["projectRoot"].as_str().expect("project root"));
        let mut expected = Vec::new();
        if phase != "repair" {
            expected.push(count_mismatch(root, "src/External.vue", 3));
        }
        expected.push(count_mismatch(root, "src/Local.vue", 4));
        assert_eq!(
            diagnostics, &expected,
            "complete exact diagnostic vector: {observation:#}"
        );
        assert_eq!(observation["exitCode"], 1);
        assert_eq!(observation["success"], false);
    }
}

fn observe(input: &Value, legacy: bool, phase: &str) -> Value {
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
    let result = match checker.scan_project().and_then(|_| checker.check_project()) {
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
    drop(checker);
    std::fs::remove_dir_all(&root)
        .expect("remove source-only temporary project after native completion");
    result
}

#[test]
fn legacy_external_callbacks_and_partial_declarations_keep_the_loose_surface() {
    assert!(
        std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none()
            || std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_none(),
        "required native compatibility control cannot be disabled"
    );
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let input: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/legacy-external-component-props/loose-controls.json"
    )))
    .expect("pinned loose controls");
    let observation = observe(&input, true, "compatibility");
    let capture = json!({
        "schema":"vize.legacy-external-props-compatibility-observation.v1",
        "scope":"Synthetic declarations. MixedCallbacks.vue is a raw remaining-gap control, not evidence its wrong count is checked.",
        "input":input, "observation":observation,
    });
    let whole = serde_json::to_string_pretty(&capture).expect("complete compatibility API capture");
    if let Some(root) = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR") {
        let directory = std::path::PathBuf::from(root).join("legacy-external-props");
        std::fs::create_dir_all(&directory).expect("capture directory");
        std::fs::write(directory.join("compatibility-observation.json"), &whole)
            .expect("retain complete compatibility result before semantic assertions");
    }
    println!("{whole}");
    let observation = &capture["observation"];
    assert_eq!(observation["kind"], "result", "{observation:#}");
    let diagnostics = observation["diagnostics"]
        .as_array()
        .expect("full diagnostics");
    let root = Path::new(observation["projectRoot"].as_str().expect("project root"));
    let mixed = root.join("src/MixedCallbacks.vue");
    let mut qualified = Vec::new();
    for diagnostic in diagnostics {
        if Path::new(diagnostic["file"].as_str().expect("diagnostic path")) != mixed {
            qualified.push(diagnostic.clone());
        }
    }
    assert_eq!(
        qualified,
        vec![count_mismatch(root, "src/Shadow.vue", 3)],
        "complete diagnostic vector for qualified compatibility files: {observation:#}"
    );
    // Do not pin MixedCallbacks.vue to a false-clean result. Its full result
    // remains visible and may improve when callback ownership is qualified.
}

fn count_mismatch(root: &Path, file: &str, line: u32) -> Value {
    json!({
        "file":root.join(file), "line":line, "column":25,
        "code":2322, "severity":1, "blockType":"Some(Template)",
        "message":"Type 'string' is not assignable to type 'number'.",
    })
}
