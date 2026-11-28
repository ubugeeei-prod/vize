use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use super::real_vue::create_project_case;

/// While `<script setup>` has a syntax error, the `useTemplateRef` calls of
/// the parse the analyzer recovers keep their template types. The recovered
/// parse cannot show every call or function setup runs, so no key is checked.
#[test]
fn template_refs_stay_inferred_while_the_script_has_a_syntax_error() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = create_project_case(
        "template-ref-parse-recovery",
        &[
            (
                "tsconfig.json",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/template-ref-parse-recovery/tsconfig.json.txt"
                ),
            ),
            (
                "src/App.vue",
                include_str!(
                    "../../../../../../../tests/_fixtures/differential/typechecker/template-ref-parse-recovery/App.vue.txt"
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
            ("src/App.vue", Some(1003), "8:1:error Identifier expected.",),
            (
                "src/App.vue",
                Some(2322),
                "5:7:error Type 'HTMLInputElement | null' is not assignable to type 'number'.\n  Type 'null' is not assignable to type 'number'.",
            ),
        ]
    );
}
