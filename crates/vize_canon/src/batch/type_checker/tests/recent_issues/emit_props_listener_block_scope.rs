//! A listener callback that reads a stable `v-for` alias is checked inside
//! its branch guard block. When it was the first listener of its component,
//! the component's inferred emit props were declared inside that block while
//! the later sibling listeners outside it still read them. The name did not
//! resolve (TS2304 on an unmapped line, so never reported), the sibling's
//! payload became `any`, and its real payload error was lost: `$event` went
//! unchecked and an arrow parameter was reported as implicitly `any` instead.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

const GENERIC_CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "emit-props-listener-block-scope/GenericChild.vue.txt"
));
const CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "emit-props-listener-block-scope/Child.vue.txt"
));
const PROBE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "emit-props-listener-block-scope/Probe.vue.txt"
));

#[test]
fn sibling_listeners_after_a_guarded_callback_keep_their_payload() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "emit-props-listener-block-scope",
        &[
            ("src/GenericChild.vue", GENERIC_CHILD),
            ("src/Child.vue", CHILD),
            ("src/Probe.vue", PROBE),
        ],
    );
    let actual = snapshot_project_diagnostics(&project);
    let _ = std::fs::remove_dir_all(project);
    // The exact rows `vue-tsc` 3.3.14 reports for the same project.
    let number_to_string =
        "Argument of type 'number' is not assignable to parameter of type 'string'.";
    let string_to_number =
        "Argument of type 'string' is not assignable to parameter of type 'number'.";
    let row = |position: &str, message: &str| {
        (
            vize_l0::String::from("src/Probe.vue"),
            Some(2345),
            vize_l0::cstr!("{position}:error {message}"),
        )
    };
    assert_eq!(
        actual,
        Some(vec![
            row("13:98", number_to_string),
            row("14:105", number_to_string),
            row("15:90", string_to_number),
            row("16:101", string_to_number),
        ])
    );
}
