//! Scoped-slot parameters are stable callback-local bindings, so their branch
//! narrowing remains valid inside an authored listener callback (#12593).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

const LIST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "scoped-slot-listener-narrowing-12593/list.vue.txt"
));
const CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "scoped-slot-listener-narrowing-12593/child.vue.txt"
));
const REPRO: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "scoped-slot-listener-narrowing-12593/repro.vue.txt"
));

#[test]
fn scoped_slot_alias_stays_narrowed_in_listener_callback() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "scoped-slot-listener-narrowing-12593",
        &[
            ("src/list.vue", LIST),
            ("src/child.vue", CHILD),
            ("src/repro.vue", REPRO),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    assert_eq!(actual, Some(Vec::new()));
}
