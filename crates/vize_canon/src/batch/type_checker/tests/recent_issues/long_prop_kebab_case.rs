//! Long camel-case props must stay typed in listener-bearing components (#10423).

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_l0::String;

macro_rules! fixture {
    ($file:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/long-prop-kebab-case-10423/",
            $file,
            ".txt"
        ))
    };
}

#[test]
fn long_prop_names_with_listeners_keep_static_and_async_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project = super::real_vue::create_project_case(
        "long-prop-kebab-case-10423",
        &[
            ("tsconfig.json", fixture!("tsconfig.json")),
            ("src/Child.vue", fixture!("src/Child.vue")),
            ("src/App.vue", fixture!("src/App.vue")),
            ("src/Invalid.vue", fixture!("src/Invalid.vue")),
        ],
    );
    assert_eq!(
        snapshot_project_diagnostics(&project),
        Some(vec![
            (
                String::from("src/Invalid.vue"),
                Some(2322),
                String::from("7:11:error Type 'string' is not assignable to type 'number'.")
            ),
            (
                String::from("src/Invalid.vue"),
                Some(2322),
                String::from("8:16:error Type 'string' is not assignable to type 'number'.")
            ),
            (
                String::from("src/Invalid.vue"),
                Some(2339),
                String::from("9:91:error Property 'missing' does not exist on type 'number'.")
            ),
        ]),
    );
    std::fs::remove_file(project.join("src/Invalid.vue")).unwrap();
    assert_eq!(snapshot_project_diagnostics(&project), Some(vec![]));
    std::fs::remove_dir_all(project).unwrap();
}
