//! Normal batch diagnostics for dollar-named lexical slot bindings (#6879).

use super::super::{BatchTypeChecker, relative_path, resolve_test_tsgo_binary};
use crate::batch::{BatchTypeCheckerOptions, TypeChecker};
use crate::virtual_ts::VirtualTsOptions;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

const PANEL: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879/Panel.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879/case.tsconfig.json.txt"
);
const VALID: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879/valid.vue.txt"
);
const WRONG: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879/wrong.vue.txt"
);
const ESCAPE: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879/escape.vue.txt"
);
const FOREIGN: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879/foreign.vue.txt"
);
const GLOBALS: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879/globals.vue.txt"
);

const LOOP_SOURCE: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879/loop-source.vue.txt"
);

#[test]
fn lexical_dollar_slots_preserve_complete_native_diagnostics_and_repairs() {
    let Some(native) = resolve_test_tsgo_binary() else {
        return;
    };
    let custody: Value = serde_json::from_str(include_str!("../../../../../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879/inputs.json")).unwrap();
    let complete_inputs: Vec<_> = [
        ("Panel.vue.txt", PANEL), ("valid.vue.txt", VALID), ("wrong.vue.txt", WRONG),
        ("escape.vue.txt", ESCAPE), ("foreign.vue.txt", FOREIGN), ("globals.vue.txt", GLOBALS),
        ("case.tsconfig.json.txt", CONFIG), ("loop-source.vue.txt", LOOP_SOURCE),
    ].into_iter().map(|(path,source)|json!({"path":path,"byteLength":source.len(),"sha256":digest(source.as_bytes())})).collect();
    assert_eq!(json!(complete_inputs), custody["inputs"]);
    let binary_bytes = std::fs::read(&native).expect("actual executable bytes");
    let mut observations = Vec::new();
    for (name, source, strict) in [
        ("valid-strict", VALID, true),
        ("wrong-strict", WRONG, true),
        ("wrong-permissive", WRONG, false),
        ("escape-strict", ESCAPE, true),
        ("foreign-strict", FOREIGN, true),
        ("loop-source-strict", LOOP_SOURCE, true),
        ("globals-strict", GLOBALS, true),
        ("globals-permissive", GLOBALS, false),
        ("repaired-strict", VALID, true),
    ] {
        let root = super::real_vue::create_project_case(
            name,
            &[
                ("src/Panel.vue", PANEL),
                ("src/App.vue", source),
                ("tsconfig.json", CONFIG),
            ],
        );
        let manifest = root.join("node_modules/vue/package.json");
        assert!(
            manifest.is_file() && root.join("node_modules/vue").is_symlink(),
            "genuine installed Vue is required, never a test facade"
        );
        let options = BatchTypeCheckerOptions {
            tsconfig_path: Some(root.join("tsconfig.json")),
            virtual_ts_options: VirtualTsOptions {
                strict_instance_globals: strict,
                ..Default::default()
            },
        };
        let mut checker = BatchTypeChecker::with_options(&root, options)
            .expect("normal configured batch factory");
        checker.set_server_count(Some(1));
        checker.scan_project().expect("actual project scan");
        let result = checker
            .check_project()
            .expect("normal actual native diagnostics");
        let diagnostics: Vec<_> = result.diagnostics.iter().map(|d| json!({
            "file": relative_path(&root, &d.file), "line": d.line + 1, "column": d.column + 1,
            "severity": d.severity, "code": d.code, "message": d.message,
            "blockType": d.block_type.map(|block| block.block_name()),
        })).collect();
        let observation = json!({
            "case": name, "strictInstanceGlobals": strict, "projectRoot": root,
            "nativeBinary": native, "nativeSha256": digest(&binary_bytes),
            "vueManifestPath": manifest.canonicalize().unwrap(), "vueManifestBytes": std::fs::read_to_string(manifest).unwrap(),
            "inputs": [{"file":"src/Panel.vue","source":PANEL}, {"file":"src/App.vue","source":source}, {"file":"tsconfig.json","source":CONFIG}],
            "exitCode": result.exit_code, "success": result.success, "diagnostics": diagnostics,
        });
        observations.push(observation);
        if let Some(dir) = std::env::var_os("VIZE_LEXICAL_DOLLAR_CAPTURE_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                Path::new(&dir).join("normal-batch-results.json"),
                serde_json::to_vec_pretty(&observations).unwrap(),
            )
            .unwrap();
        }
        std::fs::remove_dir_all(root).expect("remove only own temporary fixture");
    }
    let expected_body = include_str!(
        "../../../../../../../tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879/expected.json"
    );
    assert_eq!(
        json!(expected_body.len()),
        custody["expected"]["byteLength"]
    );
    assert_eq!(
        json!(digest(expected_body.as_bytes())),
        custody["expected"]["sha256"]
    );
    let expected: Value = serde_json::from_str(expected_body).unwrap();
    for (observed, expected) in observations.iter().zip(expected.as_array().unwrap()) {
        assert_eq!(observed["case"], expected["case"]);
        assert_eq!(
            observed["diagnostics"], expected["diagnostics"],
            "complete native vector: {observed}"
        );
        assert_eq!(observed["exitCode"], expected["exitCode"]);
        assert_eq!(observed["success"], expected["success"]);
    }
    assert_eq!(observations.len(), expected.as_array().unwrap().len());
}

fn digest(bytes: &[u8]) -> vize_l0::String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| vize_l0::cstr!("{byte:02x}"))
        .collect()
}
