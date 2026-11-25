use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::real_vue::create_project_case;

/// A `.prop` binding sets a DOM property of the mounted element, so strict
/// templates check it against the element's DOM interface rather than its
/// attribute type: `srcObject` exists on `HTMLVideoElement` but in no
/// attribute type. The value is still checked, and an undeclared property is
/// still reported as unknown. An element inside `<svg>` resolves through the
/// SVG tag-name map, where `<a>`'s `href` is a read-only `SVGAnimatedString`.
#[test]
fn prop_bindings_check_the_element_dom_properties() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "dom-property-bindings",
        &[
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/dom-property-bindings/tsconfig.json.txt"
                ),
            ),
            (
                "src/App.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/dom-property-bindings/App.vue.txt"
                ),
            ),
        ],
    );
    let diagnostics = snapshot_project_diagnostics(&root).unwrap();
    std::fs::remove_dir_all(root).unwrap();
    let diagnostics: Vec<_> = diagnostics
        .iter()
        .map(|(file, code, message)| (file.as_str(), *code, message.as_str()))
        .collect();
    assert_eq!(
        diagnostics,
        vec![
            (
                "src/App.vue",
                Some(2322),
                "10:12:error Type 'string' is not assignable to type 'SVGAnimatedString'.",
            ),
            (
                "src/App.vue",
                Some(2322),
                "11:11:error Type '1' is not assignable to type 'MediaProvider | null'.",
            ),
            (
                "src/App.vue",
                Some(2353),
                "12:11:error Object literal may only specify known properties, and '\"notAProp\"' does not exist in type 'Partial<HTMLVideoElement>'.",
            ),
        ]
    );
}
