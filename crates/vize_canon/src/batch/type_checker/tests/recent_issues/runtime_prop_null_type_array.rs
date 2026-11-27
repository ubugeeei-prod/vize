//! `null` in a runtime prop's type array is a member of the prop's type, as
//! Vue infers it: `type: [String, null]` accepts `string | null`, and every
//! other value is still checked against that union.

use super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

macro_rules! reported {
    ($file:literal) => {
        include_str!(concat!(
            "../../../../../../../tests/_fixtures/differential/typechecker/runtime-prop-null-type-array/",
            $file
        ))
    };
}

type Diagnostic = (vize_carton::String, Option<u32>, vize_carton::String);

#[test]
fn null_in_a_runtime_type_array_is_part_of_the_prop_type() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    for (name, parent, expected) in [
        (
            "nullable-values-accepted",
            reported!("UseNullableProp.vue.txt"),
            Vec::new(),
        ),
        (
            "other-values-still-checked",
            reported!("invalid.UseNullableProp.vue.txt"),
            vec![
                (
                    "UseNullableProp.vue".into(),
                    Some(2322),
                    "5:18:error Type 'number' is not assignable to type 'string'.".into(),
                ),
                (
                    "UseNullableProp.vue".into(),
                    Some(2322),
                    "5:31:error Type 'string' is not assignable to type 'number'.".into(),
                ),
            ],
        ),
    ] {
        let root = create_project_case(
            vize_carton::cstr!("runtime-prop-null-type-array-{name}").as_str(),
            &[
                ("tsconfig.json", reported!("tsconfig.json.txt")),
                ("NullableProp.vue", reported!("NullableProp.vue.txt")),
                ("UseNullableProp.vue", parent),
            ],
        );
        let snapshot = snapshot_project_diagnostics(&root);
        let _ = std::fs::remove_dir_all(&root);
        let expected: Vec<Diagnostic> = expected;
        assert_eq!(snapshot.unwrap(), expected, "{name}");
    }
}
