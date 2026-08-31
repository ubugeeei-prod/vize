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
    let archive = archive_path(&root, "vim-vize-extension.tar.gz");
    editor_archive::assert_size(&archive, "Vim archive", 2_000, 200_000)?;
    let mut entries = editor_archive::list_tar_gz(&archive)?;
    entries.sort();
    editor_archive::assert_unique(&entries, "Vim archive")?;
    editor_archive::assert_safe_entries(&entries, "Vim archive", "vim")?;
    let required = [
        "vim/LICENSE",
        "vim/README.md",
        "vim/autoload/vize.vim",
        "vim/ftdetect/vize.vim",
        "vim/plugin/vize.vim",
        "vim/test/vize_e2e_expected.vim",
        "vim/test/vize_e2e_spec.vim",
        "vim/test/vize_spec.vim",
    ];
    editor_archive::require_entries(
        &archive,
        &entries,
        &required,
        editor_archive::read_tar_text,
        "Vim archive",
    )?;
    assert_allowed(
        &entries,
        &[
            "vim/",
            "vim/LICENSE",
            "vim/README.md",
            "vim/autoload/",
            "vim/autoload/vize.vim",
            "vim/ftdetect/",
            "vim/ftdetect/vize.vim",
            "vim/plugin/",
            "vim/plugin/vize.vim",
            "vim/test/",
            "vim/test/vize_e2e_expected.vim",
            "vim/test/vize_e2e_spec.vim",
            "vim/test/vize_spec.vim",
        ],
        "Vim archive",
    )?;
    assert_no_forbidden(
        &entries,
        "Vim archive",
        &[
            ".git",
            ".github/",
            "node_modules/",
            "target/",
            ".DS_Store",
            ".tar.gz",
            "~",
        ],
    )?;
    let autoload = editor_archive::read_tar_text(&archive, "vim/autoload/vize.vim")?;
    let ftdetect = editor_archive::read_tar_text(&archive, "vim/ftdetect/vize.vim")?;
    let spec = editor_archive::read_tar_text(&archive, "vim/test/vize_spec.vim")?;
    for (source, needle) in [
        (&autoload, r#"'cmd': ['vize', 'lsp']"#),
        (&autoload, r#"'allowlist': ['vue', 'art-vue']"#),
        (&autoload, "initialization_options"),
        (&autoload, "function! vize#vim_lsp_config"),
        (&autoload, "lsp#register_server"),
        (&ftdetect, "*.vue setlocal filetype=vue"),
        (&ftdetect, "*.art.vue call <SID>detect_art_vue()"),
        (&ftdetect, "setlocal filetype=art-vue"),
        (&ftdetect, "if empty(&l:syntax)"),
        (&spec, "vize#vim_lsp_config"),
        (&spec, "assert_equal(['vize', 'lsp']"),
    ] {
        editor_archive::expect_contains(source, needle, &format!("Vim package missing {needle}"))?;
    }
    println!(
        "Vim package smoke passed: {} ({} entries)",
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
