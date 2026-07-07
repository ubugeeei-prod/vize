use super::vscode_extension_is_installed;

#[test]
fn vscode_status_accepts_published_extension_id() {
    assert!(vscode_extension_is_installed(
        "ms-vscode.cpptools\nubugeeei.vize\n"
    ));
}

#[test]
fn vscode_status_accepts_legacy_extension_id() {
    assert!(vscode_extension_is_installed("vize.vize\n"));
}

#[test]
fn vscode_status_requires_exact_extension_id() {
    assert!(!vscode_extension_is_installed("some.ubugeeei.vize-plus\n"));
}

#[test]
fn vscode_status_trims_lines_and_ignores_case() {
    assert!(vscode_extension_is_installed("  UBUGEeEI.VIZE  \r\n"));
    assert!(vscode_extension_is_installed("\tvize.vize\t\n"));
}

#[test]
fn vscode_status_rejects_empty_and_partial_lines() {
    assert!(!vscode_extension_is_installed(""));
    assert!(!vscode_extension_is_installed("ubugeeei.vize-extra\n"));
    assert!(!vscode_extension_is_installed("prefix ubugeeei.vize\n"));
}
