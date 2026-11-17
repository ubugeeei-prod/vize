use crate::support::fix_history_runner::check_pack;

#[test]
fn recoverable_template_repair_and_vfor_guarded_component_props() {
    check_pack(
        "recoverable-template-vfor-guard-props-1566",
        "fix_history_cases::recoverable_template_vfor_guard_props_1566::recoverable_template_repair_and_vfor_guarded_component_props",
        "beeb9d20faa6f952a8137b60f531cc37088deccd",
        Some(1566),
        &[
            "recoverable-repair-keeps-template-checked",
            "recoverable-repair-keeps-dependent-diagnostic",
            "recoverable-repair-template-first-keeps-dependent-diagnostic",
            "vfor-component-props-narrowed-by-enclosing-vif",
            "vfor-component-props-without-enclosing-vif",
        ],
    );
}
