//! Whole fixed-revision SFCs from the four failing 42a Real Project Matrix cases.
//! The corpus manifest retains their original revisions, licenses and byte hashes.
#![expect(
    clippy::unwrap_used,
    reason = "fixture formatting failures must fail tests"
)]

use vize_glyph::{FormatOptions, format_sfc};

fn assert_whole_sfc_fixed_point(id: &str, source: &str) {
    let options = FormatOptions::default();
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();
    let third = format_sfc(&second.code, &options).unwrap();

    assert_eq!(
        first.changed,
        first.code.as_str() != source,
        "{id}: first-pass changed must report actual whole-file byte changes"
    );
    assert_eq!(
        first.code, second.code,
        "{id}: the complete second output must equal the complete first output"
    );
    assert_eq!(
        second.code, third.code,
        "{id}: the complete third output must retain the same fixed point"
    );
    assert!(!second.changed, "{id}: second pass must report unchanged");
    assert!(!third.changed, "{id}: third pass must report unchanged");
}

#[test]
fn lew_ui_desc_engine_whole_sfc_is_idempotent() {
    assert_whole_sfc_fixed_point(
        "lew-ui/docs/views/desc-engine/index.vue",
        include_str!(
            "../../../tests/_fixtures/differential/formatter-regressions/sequence-directive-indent-8023/lew-ui/docs/views/desc-engine/index.vue.txt"
        ),
    );
}

#[test]
fn lew_ui_form_engine_whole_sfc_is_idempotent() {
    assert_whole_sfc_fixed_point(
        "lew-ui/docs/views/form-engine/index.vue",
        include_str!(
            "../../../tests/_fixtures/differential/formatter-regressions/sequence-directive-indent-8023/lew-ui/docs/views/form-engine/index.vue.txt"
        ),
    );
}

#[test]
fn lew_ui_tree_select_whole_sfc_is_idempotent() {
    assert_whole_sfc_fixed_point(
        "lew-ui/lib/components/form/tree-select/src/LewTreeSelect.vue",
        include_str!(
            "../../../tests/_fixtures/differential/formatter-regressions/sequence-directive-indent-8023/lew-ui/lib/components/form/tree-select/src/LewTreeSelect.vue.txt"
        ),
    );
}

#[test]
fn frappe_crm_form_builder_panel_whole_sfc_is_idempotent() {
    assert_whole_sfc_fixed_point(
        "frappe-crm/frontend/src/components/Settings/Forms/FormBuilderPanel.vue",
        include_str!(
            "../../../tests/_fixtures/differential/formatter-regressions/sequence-directive-indent-8023/frappe-crm/frontend/src/components/Settings/Forms/FormBuilderPanel.vue.txt"
        ),
    );
}
