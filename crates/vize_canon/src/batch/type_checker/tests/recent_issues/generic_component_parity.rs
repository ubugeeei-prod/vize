//! Generic-component parity cases retained from #11487.

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::cstr;

const CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-component-parity-11487/Child.vue.txt"
));
const UNSHADOWED_CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-component-parity-11487/UnshadowedChild.vue.txt"
));
const PARENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-component-parity-11487/Parent.vue.txt"
));
const GENERIC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-component-parity-11487/Generic.vue.txt"
));
const CONSUMER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-component-parity-11487/consumer.ts.txt"
));

#[test]
fn optional_define_model_rejects_a_non_optional_parent_target() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = create_project_case(
        "optional-generic-model-writeback-11487",
        &[("src/Child.vue", CHILD), ("src/Parent.vue", PARENT)],
    );
    let actual = snapshot_project_diagnostics(&project).expect("native diagnostics");
    let _ = std::fs::remove_dir_all(project);

    assert_eq!(
        actual,
        vec![(
            cstr!("src/Parent.vue"),
            Some(2322_u32),
            cstr!(
                "7:32:error Type 'any[] | undefined' is not assignable to type 'number[]'.\n\
                 Type 'undefined' is not assignable to type 'number[]'.",
            ),
        )]
    );
}

#[test]
fn generated_define_model_rejects_a_non_optional_parent_target() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let parent = PARENT.replace("./Child.vue", "./UnshadowedChild.vue");
    let project = super::real_vue::create_project_case(
        "unshadowed-optional-generic-model-writeback-11487",
        &[
            ("src/UnshadowedChild.vue", UNSHADOWED_CHILD),
            ("src/Parent.vue", &parent),
        ],
    );
    let actual = snapshot_project_diagnostics(&project).expect("native diagnostics");
    let _ = std::fs::remove_dir_all(project);

    assert_eq!(
        actual,
        vec![(
            cstr!("src/Parent.vue"),
            Some(2322_u32),
            cstr!(
                "7:32:error Type 'any[] | undefined' is not assignable to type 'number[]'.\n\
                 Type 'undefined' is not assignable to type 'number[]'.",
            ),
        )]
    );
}

#[test]
fn instance_type_rejects_a_generic_sfc_function() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = create_project_case(
        "generic-component-instance-type-11487",
        &[("src/Generic.vue", GENERIC), ("src/consumer.ts", CONSUMER)],
    );
    let actual = snapshot_project_diagnostics(&project).expect("native diagnostics");
    let _ = std::fs::remove_dir_all(project);

    assert_eq!(actual.len(), 1, "{actual:#?}");
    assert_eq!(actual[0].0, "src/consumer.ts");
    assert_eq!(actual[0].1, Some(2344));
}
