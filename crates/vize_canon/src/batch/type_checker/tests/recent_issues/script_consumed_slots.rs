use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::real_vue::create_project_case;

const CHILD: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/typechecker/script-consumed-slots-13578/Child.vue.txt"
);
const PARENT: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/typechecker/script-consumed-slots-13578/Parent.vue.txt"
);

#[test]
fn script_consumed_slots_keep_public_inference_open() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "script-consumed-slots",
        &[("src/Child.vue", CHILD), ("src/Parent.vue", PARENT)],
    );
    let diagnostics = snapshot_project_diagnostics(&root);
    std::fs::remove_dir_all(root).unwrap();
    assert_eq!(diagnostics, Some(Vec::new()));
}

#[test]
fn script_consumed_slots_preserve_template_payload_and_explicit_contracts() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    for (name, child, parent, code) in [
        (
            "explicit",
            CHILD.replace(
                "</script>",
                "defineSlots<{ 'submit-status'(props: { value: string }): unknown }>()\n</script>",
            ),
            PARENT.to_owned(),
            2339,
        ),
        (
            "inferred-payload",
            CHILD.to_owned(),
            PARENT.replace(
                "#field-label=\"{ field }\">{{ field }}",
                "#submit-status=\"{ value }\">{{ value.toFixed() }}",
            ),
            2551,
        ),
    ] {
        let root = create_project_case(
            name,
            &[("src/Child.vue", &child), ("src/Parent.vue", &parent)],
        );
        let diagnostics = snapshot_project_diagnostics(&root).unwrap();
        std::fs::remove_dir_all(root).unwrap();
        let codes: Vec<_> = diagnostics
            .into_iter()
            .map(|(file, code, _)| (file, code))
            .collect();
        assert_eq!(codes, vec![("src/Parent.vue".into(), Some(code))], "{name}");
    }
}

#[test]
fn inferred_dollar_slots_keep_script_use_slots_exact() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "typed-use-slots",
        &[
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/typed-use-slots-13578/tsconfig.json.txt"
                ),
            ),
            (
                "src/Child.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/typed-use-slots-13578/Child.vue.txt"
                ),
            ),
            (
                "src/Parent.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/typed-use-slots-13578/Parent.vue.txt"
                ),
            ),
        ],
    );
    let diagnostics = snapshot_project_diagnostics(&root).unwrap();
    std::fs::remove_dir_all(root).unwrap();
    let codes: Vec<_> = diagnostics
        .into_iter()
        .map(|(file, code, _)| (file, code))
        .collect();
    assert_eq!(codes, vec![("src/Parent.vue".into(), Some(2339))]);
}
