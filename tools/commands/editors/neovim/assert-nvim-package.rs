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
    let archive = archive_path(&root, "nvim-vize-extension.tar.gz");
    editor_archive::assert_size(&archive, "Neovim archive", 2_000, 200_000)?;
    let mut entries = editor_archive::list_tar_gz(&archive)?;
    entries.sort();
    editor_archive::assert_unique(&entries, "Neovim archive")?;
    editor_archive::assert_safe_entries(&entries, "Neovim archive", "nvim")?;

    let required = [
        "nvim/LICENSE",
        "nvim/README.md",
        "nvim/ftdetect/vize.lua",
        "nvim/lua/vize/config.lua",
        "nvim/lua/vize/init.lua",
        "nvim/plugin/vize.lua",
        "nvim/test/component_contract_hover.lua",
        "nvim/test/ref_surface_hover.lua",
        "nvim/test/vize_e2e_expected.lua",
        "nvim/test/vize_e2e_spec.lua",
        "nvim/test/vize_spec.lua",
    ];
    editor_archive::require_entries(
        &archive,
        &entries,
        &required,
        editor_archive::read_tar_text,
        "Neovim archive",
    )?;
    assert_allowed(
        &entries,
        &[
            "nvim/",
            "nvim/LICENSE",
            "nvim/README.md",
            "nvim/ftdetect/",
            "nvim/ftdetect/vize.lua",
            "nvim/lua/",
            "nvim/lua/vize/",
            "nvim/lua/vize/config.lua",
            "nvim/lua/vize/init.lua",
            "nvim/plugin/",
            "nvim/plugin/vize.lua",
            "nvim/test/",
            "nvim/test/component_contract_hover.lua",
            "nvim/test/ref_surface_hover.lua",
            "nvim/test/vize_e2e_expected.lua",
            "nvim/test/vize_e2e_spec.lua",
            "nvim/test/vize_spec.lua",
        ],
        "Neovim archive",
    )?;
    assert_no_forbidden(
        &entries,
        "Neovim archive",
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

    let config = editor_archive::read_tar_text(&archive, "nvim/lua/vize/config.lua")?;
    let init = editor_archive::read_tar_text(&archive, "nvim/lua/vize/init.lua")?;
    let ftdetect = editor_archive::read_tar_text(&archive, "nvim/ftdetect/vize.lua")?;
    let spec = editor_archive::read_tar_text(&archive, "nvim/test/vize_spec.lua")?;
    let e2e = editor_archive::read_tar_text(&archive, "nvim/test/vize_e2e_spec.lua")?;
    for (source, needle) in [
        (&config, r#"cmd = { "vize", "lsp" }"#),
        (&config, r#"filetypes = { "vue", "art-vue" }"#),
        (
            &config,
            r#"root_markers = { "vize.config.pkl", "vize.config.json", "package.json", ".git" }"#,
        ),
        (&config, "lint = true"),
        (&config, "recommended = {"),
        (&config, "init_options = profiles.recommended"),
        (&config, r#"assert_list("cmd""#),
        (&init, r#"vim.lsp.config("vize", resolved)"#),
        (&init, r#"vim.lsp.enable("vize")"#),
        (&ftdetect, r#"pattern = "*.vue""#),
        (&ftdetect, r#"pattern = "*.art.vue""#),
        (&ftdetect, r#"filetype = "art-vue""#),
        (&ftdetect, r#"vim.treesitter.language.add("art-vue")"#),
        (
            &ftdetect,
            r#"vim.treesitter.language.register("vue", "art-vue")"#,
        ),
        (&spec, "config.normalize"),
        (&spec, "vim.lsp.config.vize"),
        (&e2e, "vim.lsp.start"),
        (&e2e, "textDocument/completion"),
        (&e2e, "textDocument/hover"),
        (&e2e, "textDocument/codeAction"),
        (&e2e, "textDocument/formatting"),
        (&e2e, "textDocument/semanticTokens/full"),
        (&e2e, "textDocument/rename"),
    ] {
        editor_archive::expect_contains(
            source,
            needle,
            &format!("Neovim package missing {needle}"),
        )?;
    }

    println!(
        "Neovim package smoke passed: {} ({} entries)",
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
