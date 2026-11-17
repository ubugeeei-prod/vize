use crate::support::fix_history_runner::check_pack;

#[test]
fn sfc_block_text_inside_script_strings_is_not_hoisted() {
    check_pack(
        "sfc-blocks-in-script-strings-2200",
        "fix_history_cases::sfc_blocks_in_script_strings_2200::sfc_block_text_inside_script_strings_is_not_hoisted",
        "53c5d6e507b52decb903214322ab56e0312dbec0",
        Some(2200),
        &["template-literal-sfc-sample", "real-import-still-resolved"],
    );
}
