//! Generic setup locals retain Vue's distributive `Ref` unwrap even when the
//! local itself is constrained to a plain object (#11589).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

const APP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-local-indexing-11589/App.vue.txt"
));
const TYPES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-local-indexing-11589/types.ts.txt"
));
const TSCONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-local-indexing-11589/tsconfig.json.txt"
));

#[test]
fn generic_setup_local_indexing_reports_the_vue_ref_distribution_error() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project = super::real_vue::create_project_case(
        "generic-local-indexing-11589",
        &[
            ("src/App.vue", APP),
            ("src/types.ts", TYPES),
            ("tsconfig.json", TSCONFIG),
        ],
    );

    let actual = snapshot_project_diagnostics(&project).expect("native diagnostics");
    let _ = std::fs::remove_dir_all(project);

    assert_eq!(
        actual.len(),
        1,
        "expected only the vue-tsc parity diagnostic: {actual:#?}"
    );
    assert_eq!(actual[0].0, "src/App.vue");
    assert_eq!(actual[0].1, Some(2536));
    // The native printer preserves the alias in isolation and expands the
    // same conditional when the complete checker suite has instantiated it.
    // Keep both complete packets explicit; neither is a partial oracle.
    assert!(
        matches!(
            actual[0].2.as_str(),
            "6:17:error Type 'K' cannot be used to index type '__UN<T>'."
                | "6:17:error Type 'K' cannot be used to index type 'T extends Ref<infer __V, infer __V> ? __V : T'."
        ),
        "unexpected generic local indexing diagnostic: {actual:#?}"
    );
}
