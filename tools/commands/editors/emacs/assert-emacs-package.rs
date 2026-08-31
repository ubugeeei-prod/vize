#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```

use std::{collections::BTreeSet, env, path::PathBuf, process::ExitCode};

#[path = "../../../rust/common.rs"]
mod common;
#[path = "../../../rust/editor_archive.rs"]
mod editor_archive;

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let root = common::repo_root()?;
    let archive = archive_path(&root, "emacs-vize-extension.tar.gz");
    editor_archive::assert_size(&archive, "Emacs archive", 2_000, 200_000)?;
    let mut entries = editor_archive::list_tar_gz(&archive)?;
    entries.sort();
    editor_archive::assert_unique(&entries, "Emacs archive")?;
    editor_archive::assert_safe_entries(&entries, "Emacs archive", "emacs")?;
    let required = [
        "emacs/LICENSE",
        "emacs/README.md",
        "emacs/test/vize-test.el",
        "emacs/vize.el",
    ];
    editor_archive::require_entries(
        &archive,
        &entries,
        &required,
        editor_archive::read_tar_text,
        "Emacs archive",
    )?;
    assert_allowed(
        &entries,
        &[
            "emacs/",
            "emacs/LICENSE",
            "emacs/README.md",
            "emacs/test/",
            "emacs/test/vize-test.el",
            "emacs/vize.el",
        ],
        "Emacs archive",
    )?;
    assert_no_forbidden(
        &entries,
        "Emacs archive",
        &[
            ".git",
            ".github/",
            "node_modules/",
            "target/",
            ".DS_Store",
            ".elc",
            ".tar.gz",
            "~",
        ],
    )?;
    let vize_el = editor_archive::read_tar_text(&archive, "emacs/vize.el")?;
    let test_el = editor_archive::read_tar_text(&archive, "emacs/test/vize-test.el")?;
    for (source, needle) in [
        (&vize_el, "lexical-binding: t"),
        (&vize_el, r#"defcustom vize-eglot-command '("vize" "lsp")"#),
        (&vize_el, "defcustom vize-eglot-profile 'recommended"),
        (
            &vize_el,
            "recommended . (:editor t :ecosystem t :lint t :typecheck t)",
        ),
        (&vize_el, "define-derived-mode vize-vue-mode"),
        (&vize_el, "define-derived-mode vize-art-vue-mode"),
        (&vize_el, ":initializationOptions options"),
        (&vize_el, "eglot-server-programs"),
        (&vize_el, "provide 'vize"),
        (&test_el, "ert-deftest vize-eglot-default-program"),
        (&test_el, ":initializationOptions"),
        (&test_el, "ert-deftest vize-eglot-off-program"),
    ] {
        editor_archive::expect_contains(
            source,
            needle,
            &format!("Emacs package missing {needle}"),
        )?;
    }
    println!(
        "Emacs package smoke passed: {} ({} entries)",
        common::relative_path(&root, &archive),
        entries.len()
    );
    Ok(())
}

fn archive_path(root: &std::path::Path, default: &str) -> PathBuf {
    env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join(default))
}

fn assert_allowed(entries: &[String], allowed: &[&str], label: &str) -> Result<(), String> {
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    for entry in entries {
        if !allowed.contains(entry.as_str()) {
            return Err(format!("{label} ships an unexpected file: {entry}"));
        }
    }
    Ok(())
}

fn assert_no_forbidden(entries: &[String], label: &str, forbidden: &[&str]) -> Result<(), String> {
    for entry in entries {
        for pattern in forbidden {
            if entry.contains(pattern) || entry.ends_with(pattern) {
                return Err(format!("{label} must not ship {entry}"));
            }
        }
    }
    Ok(())
}
