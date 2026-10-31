//! Actual per-usage constructor selection is also the props authority (#11023).

use super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_l0::String;

const COMPONENTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/constructor-props-correlation-11023/components.ts.txt"
));
const VALID: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/constructor-props-correlation-11023/Valid.vue.txt"
));
const INVALID: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/constructor-props-correlation-11023/Invalid.vue.txt"
));

#[test]
fn constructor_props_follow_each_usage_in_both_overload_orders() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::super::real_vue::create_project_case(
        "constructor-props-correlation-11023",
        &[
            ("src/components.ts", COMPONENTS),
            ("src/Valid.vue", VALID),
            ("src/Invalid.vue", INVALID),
        ],
    );
    assert_eq!(
        snapshot_project_diagnostics(&project),
        Some(vec![
            (
                String::from("src/Invalid.vue"),
                Some(2339),
                String::from("5:50:error Property 'toUpperCase' does not exist on type 'number'."),
            ),
            (
                String::from("src/Invalid.vue"),
                Some(2551),
                String::from(
                    "6:55:error Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?"
                ),
            ),
        ]),
        "each usage must keep its exact numeric/string slots and props, including generic constructors",
    );
    let repaired = INVALID.replace("toUpperCase()", "toFixed()").replace(
        "item.toFixed() }}</FirstZero>",
        "item.toUpperCase() }}</FirstZero>",
    );
    std::fs::write(project.join("src/Invalid.vue"), repaired).unwrap();
    assert_eq!(snapshot_project_diagnostics(&project), Some(vec![]));
    std::fs::remove_dir_all(project).unwrap();
}
