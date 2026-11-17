use crate::support::fix_history_runner::check_pack;

#[test]
fn tsx_sfc_named_exports_keep_downstream_contextual_typing() {
    check_pack(
        "tsx-sfc-named-exports-2303",
        "fix_history_cases::tsx_sfc_named_exports_2303::tsx_sfc_named_exports_keep_downstream_contextual_typing",
        "27260d211025441b6303090d4fd7acaa65528f15",
        Some(2303),
        &[
            "tsx-sfc-named-export-contextual-typing",
            "tsx-sfc-named-export-wrong-slot-return",
        ],
    );
}
