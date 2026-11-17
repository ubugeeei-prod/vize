use crate::support::fix_history_runner::check_pack;

#[test]
fn virtual_ts_keeps_default_es2022_and_dom_lib_references() {
    check_pack(
        "virtual-ts-default-lib-references-2551",
        "fix_history_cases::virtual_ts_default_lib_references_2551::virtual_ts_keeps_default_es2022_and_dom_lib_references",
        "7a80804d3f671ab2cbea238172770ccfc256e9bb",
        Some(2551),
        &[
            "default-es2022-dom-libs-without-tsconfig-dom",
            "default-libs-still-type-dom-and-es2022-members",
        ],
    );
}
