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

#[path = "../../../support/common.rs"]
mod common;
#[path = "../../../support/editors/archive.rs"]
mod editor_archive;

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let root = common::repo_root()?;
    let archive = archive_path(&root, "helix-vize-extension.tar.gz");
    editor_archive::assert_size(&archive, "Helix archive", 1_000, 100_000)?;
    let mut entries = editor_archive::list_tar_gz(&archive)?;
    entries.sort();
    editor_archive::assert_unique(&entries, "Helix archive")?;
    editor_archive::assert_safe_entries(&entries, "Helix archive", "helix")?;
    let required = ["helix/LICENSE", "helix/README.md", "helix/languages.toml"];
    editor_archive::require_entries(
        &archive,
        &entries,
        &required,
        editor_archive::read_tar_text,
        "Helix archive",
    )?;
    assert_allowed(
        &entries,
        &[
            "helix/",
            "helix/LICENSE",
            "helix/README.md",
            "helix/languages.toml",
        ],
        "Helix archive",
    )?;
    assert_no_forbidden(
        &entries,
        "Helix archive",
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
    let languages = editor_archive::read_tar_text(&archive, "helix/languages.toml")?;
    for needle in [
        "[language-server.vize]",
        r#"command = "vize""#,
        r#"args = ["lsp"]"#,
        "[language-server.vize.config]",
        "editor = true",
        "ecosystem = true",
        "lint = true",
        "typecheck = true",
        r#"name = "vue""#,
        r#"scope = "source.vue""#,
        r#"file-types = ["vue"]"#,
        r#"name = "art-vue""#,
        r#"language-id = "art-vue""#,
        r#"scope = "source.art-vue""#,
        r#"file-types = [{ glob = "*.art.vue" }]"#,
        r#"roots = ["vize.config.pkl", "vize.config.json", "package.json", ".git"]"#,
        r#"language-servers = ["vize"]"#,
    ] {
        editor_archive::expect_contains(
            &languages,
            needle,
            &format!("Helix package missing {needle}"),
        )?;
    }
    println!(
        "Helix package smoke passed: {} ({} entries)",
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
