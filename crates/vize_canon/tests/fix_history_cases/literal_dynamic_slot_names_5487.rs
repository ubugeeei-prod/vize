use crate::support::fix_history_runner::check_pack;

#[test]
fn literal_template_slot_name_uses_index_signature_payload() {
    check_pack(
        "literal-dynamic-slot-names-5487",
        "fix_history_cases::literal_dynamic_slot_names_5487::literal_template_slot_name_uses_index_signature_payload",
        "cd93f9fba56a8c067691a96f06f15eff2285bd10",
        Some(5487),
        &[
            "literal-template-slot-name-payload",
            "literal-template-slot-name-wrong-header",
        ],
    );
}
