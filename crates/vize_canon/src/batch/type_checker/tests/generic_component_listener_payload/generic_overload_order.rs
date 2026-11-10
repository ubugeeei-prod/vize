use super::*;

const CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-emit-overload-order-11435/Child.vue.txt"
));
const APP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-emit-overload-order-11435/App.vue.txt"
));

// Signatures before a generic overload must keep typed listener payloads (#11435).
#[test]
fn overloads_declared_before_a_generic_emit_keep_typed_payloads() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "generic-emit-overload-order",
        &[("src/Child.vue", CHILD), ("src/App.vue", APP)],
    );
    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };
    assert_eq!(snapshot, vec![], "matching payloads must pass");

    for (handler, column, expected, actual) in [
        ("onFirst", 11, "number", "string"),
        ("onGen", 28, "unknown", "string"),
        ("onLast", 41, "string", "number"),
    ] {
        let declared = format!("function {handler}(value: {expected})");
        assert_eq!(APP.matches(declared.as_str()).count(), 1);
        let wrong = APP.replace(
            declared.as_str(),
            &format!("function {handler}(value: {actual})"),
        );
        std::fs::write(project_root.join("src/App.vue"), &wrong).unwrap();
        let message = format!(
            "10:{column}:error Type '(value: {actual}) => void' is not assignable to type '(value: {expected}) => any'.\n\
             Types of parameters 'value' and 'value' are incompatible.\n\
             Type '{expected}' is not assignable to type '{actual}'."
        );
        assert_eq!(
            snapshot_project_diagnostics(&project_root).unwrap(),
            vec![("src/App.vue".into(), Some(2322), message.as_str().into())]
        );
    }
    let _ = std::fs::remove_dir_all(&project_root);
}
