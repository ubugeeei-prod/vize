use super::{EditorArgs, EditorOperation, editor_operation, vscode_extension_is_installed};
use crate::commands::lsp::{LspTransport, transport_from_port};

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

#[test]
fn editor_operation_defaults_to_install() {
    assert_eq!(
        editor_operation(&editor_args(false, false, false)),
        EditorOperation::Install
    );
}

#[test]
fn editor_operation_accepts_explicit_install() {
    assert_eq!(
        editor_operation(&editor_args(true, false, false)),
        EditorOperation::Install
    );
}

#[test]
fn editor_operation_prefers_uninstall_over_status_and_install() {
    assert_eq!(
        editor_operation(&editor_args(true, true, true)),
        EditorOperation::Uninstall
    );
}

#[test]
fn editor_operation_prefers_status_over_install() {
    assert_eq!(
        editor_operation(&editor_args(true, false, true)),
        EditorOperation::Status
    );
}

#[test]
fn ide_lsp_defaults_to_stdio_transport() {
    assert_eq!(transport_from_port(None), LspTransport::Stdio);
}

#[test]
fn ide_lsp_port_selects_tcp_transport() {
    assert_eq!(transport_from_port(Some(9333)), LspTransport::Tcp(9333));
}

fn editor_args(install: bool, uninstall: bool, status: bool) -> EditorArgs {
    EditorArgs {
        install,
        status,
        uninstall,
    }
}
