use super::{
    EditorArgs, EditorOperation, editor_operation, unpack_zed_extension_archive,
    validate_zed_archive_member, vscode_extension_is_installed, vscode_extension_spec,
    vscode_extension_version, zed_dev_extension_dir, zed_release_asset_url,
};
use crate::commands::lsp::{LspTransport, transport_from_port};
use flate2::{Compression, write::GzEncoder};
use std::collections::BTreeSet;
use std::io::Cursor;

#[test]
fn vscode_status_accepts_published_extension_id() {
    assert!(vscode_extension_is_installed(
        "ms-vscode.cpptools\nubugeeei.vize\n"
    ));
}

#[test]
fn vscode_status_reads_published_extension_version() {
    assert_eq!(
        vscode_extension_version("ms-vscode.cpptools@1.0.0\nubugeeei.vize@0.493.0\n"),
        Some(Some("0.493.0"))
    );
}

#[test]
fn vscode_status_reads_legacy_extension_version() {
    assert_eq!(
        vscode_extension_version("vize.vize@0.443.0\n"),
        Some(Some("0.443.0"))
    );
}

#[test]
fn vscode_status_handles_code_without_version_output() {
    assert_eq!(vscode_extension_version("ubugeeei.vize\n"), Some(None));
}

#[test]
fn vscode_install_pins_the_cli_version() {
    assert_eq!(vscode_extension_spec("0.493.0"), "ubugeeei.vize@0.493.0");
}

#[test]
fn zed_release_asset_matches_the_cli_version() {
    assert_eq!(
        zed_release_asset_url("0.493.0"),
        "https://github.com/ubugeeei-prod/vize/releases/download/v0.493.0/zed-vize-extension.tar.gz"
    );
}

#[test]
fn zed_archive_member_rejects_traversal_links_and_duplicate_paths() {
    for name in [
        b"/zed/extension.toml".as_slice(),
        b"zed/../escape".as_slice(),
        b"zed/./extension.toml".as_slice(),
        b"zed\\extension.toml".as_slice(),
        b"other/extension.toml".as_slice(),
    ] {
        assert!(
            validate_zed_archive_member(name, tar::EntryType::file(), 1, &mut BTreeSet::new())
                .is_err()
        );
    }
    for entry_type in [tar::EntryType::symlink(), tar::EntryType::hard_link()] {
        assert!(
            validate_zed_archive_member(
                b"zed/extension.toml",
                entry_type,
                0,
                &mut BTreeSet::new(),
            )
            .is_err()
        );
    }
    let mut seen = BTreeSet::new();
    assert!(
        validate_zed_archive_member(b"zed/extension.toml", tar::EntryType::file(), 1, &mut seen)
            .is_ok()
    );
    assert!(
        validate_zed_archive_member(b"zed/extension.toml", tar::EntryType::file(), 1, &mut seen)
            .is_err()
    );
}

#[test]
fn zed_archive_is_validated_before_any_member_is_extracted() {
    let archive = zed_archive([
        (
            "zed/extension.toml",
            tar::EntryType::file(),
            b"id = \"vize\"".as_slice(),
        ),
        (
            "zed/link",
            tar::EntryType::symlink(),
            b"../outside".as_slice(),
        ),
    ]);
    let destination = tempfile::tempdir().unwrap();

    assert!(unpack_zed_extension_archive(&archive, destination.path()).is_err());
    assert!(!destination.path().join("zed").exists());
}

#[test]
fn zed_archive_extracts_regular_source_beneath_the_destination() {
    let archive = zed_archive([
        ("zed/", tar::EntryType::dir(), b"".as_slice()),
        (
            "zed/extension.toml",
            tar::EntryType::file(),
            b"id = \"vize\"".as_slice(),
        ),
        (
            "zed/src/lib.rs",
            tar::EntryType::file(),
            b"pub fn run() {}".as_slice(),
        ),
    ]);
    let destination = tempfile::tempdir().unwrap();

    unpack_zed_extension_archive(&archive, destination.path()).unwrap();

    assert_eq!(
        std::fs::read_to_string(destination.path().join("zed/extension.toml")).unwrap(),
        "id = \"vize\""
    );
    assert_eq!(
        std::fs::read_to_string(destination.path().join("zed/src/lib.rs")).unwrap(),
        "pub fn run() {}"
    );
}

fn zed_archive<const N: usize>(members: [(&str, tar::EntryType, &[u8]); N]) -> Vec<u8> {
    let encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut archive = tar::Builder::new(encoder);
    for (name, entry_type, bytes) in members {
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(entry_type);
        header.set_mode(if entry_type.is_dir() { 0o755 } else { 0o644 });
        header.set_size(if entry_type.is_file() {
            bytes.len() as u64
        } else {
            0
        });
        if entry_type.is_symlink() {
            header
                .set_link_name(std::str::from_utf8(bytes).unwrap())
                .unwrap();
        }
        header.set_cksum();
        archive
            .append_data(
                &mut header,
                name,
                Cursor::new(if entry_type.is_file() { bytes } else { &[] }),
            )
            .unwrap();
    }
    archive.into_inner().unwrap().finish().unwrap()
}

#[test]
fn zed_source_is_prepared_outside_the_installed_extension_store() {
    assert_eq!(
        zed_dev_extension_dir(std::path::Path::new("/data/Zed/extensions/installed")),
        std::path::Path::new("/data/Zed/extensions/dev/vize")
    );
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
fn vscode_status_accepts_extension_id_among_crlf_lines() {
    assert!(vscode_extension_is_installed(
        "foo.bar\r\n  ubugeeei.vize  \r\nbaz.qux\r\n"
    ));
}

#[test]
fn vscode_status_rejects_empty_and_partial_lines() {
    assert!(!vscode_extension_is_installed(""));
    assert!(!vscode_extension_is_installed("ubugeeei.vize-extra\n"));
    assert!(!vscode_extension_is_installed("prefix ubugeeei.vize\n"));
}

#[test]
fn vscode_status_rejects_comments_suffixes_and_other_publishers() {
    assert!(!vscode_extension_is_installed("ubugeeei.vize # comment\n"));
    assert!(!vscode_extension_is_installed("publisher.ubugeeei.vize\n"));
    assert!(!vscode_extension_is_installed("ubugeeei.vize.preview\n"));
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
fn editor_operation_prefers_uninstall_when_install_is_false() {
    assert_eq!(
        editor_operation(&editor_args(false, true, false)),
        EditorOperation::Uninstall
    );
}

#[test]
fn editor_operation_prefers_uninstall_over_status() {
    assert_eq!(
        editor_operation(&editor_args(false, true, true)),
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
fn editor_operation_accepts_status_without_install() {
    assert_eq!(
        editor_operation(&editor_args(false, false, true)),
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
