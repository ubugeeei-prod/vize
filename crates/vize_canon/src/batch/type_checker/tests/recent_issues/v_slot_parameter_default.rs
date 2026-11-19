//! A default for the whole `v-slot` destructuring parameter must not make the
//! virtual TypeScript syntactically invalid and suppress semantic diagnostics.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

const REPRO: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/v-slot-parameter-default-13415/A.vue.txt"
));
const SEMANTIC_CONTROL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/v-slot-parameter-default-13415/semantic.ts.txt"
));
const REPORTED_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/v-slot-parameter-default-13415/tsconfig.json.txt"
));

#[test]
fn defaulted_v_slot_pattern_preserves_the_project_semantic_pass() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "v-slot-parameter-default",
        &[
            ("A.vue", REPRO),
            ("semantic.ts", SEMANTIC_CONTROL),
            ("tsconfig.json", REPORTED_CONFIG),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    let _ = std::fs::remove_dir_all(&project_root);

    assert!(
        snapshot.iter().all(|(file, code, _)| {
            file != "A.vue" || !matches!(code, Some(1003 | 1005 | 1359))
        }),
        "the slot default must not produce virtual TypeScript syntax errors: {snapshot:#?}"
    );
    assert!(
        snapshot
            .iter()
            .any(|(file, code, _)| file == "semantic.ts" && *code == Some(2322)),
        "the unrelated semantic diagnostic must remain visible: {snapshot:#?}"
    );
}
