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
    let archive = archive_path(&root, "zed-vize-extension.tar.gz");
    editor_archive::assert_size(&archive, "Zed extension archive", 5_000, 2_000_000)?;
    let mut entries = editor_archive::list_tar_gz(&archive)?;
    entries.sort();
    editor_archive::assert_unique(&entries, "Zed archive")?;
    editor_archive::assert_safe_entries(&entries, "Zed archive", "zed")?;
    let required = [
        "zed/Cargo.lock",
        "zed/Cargo.toml",
        "zed/LICENSE",
        "zed/README.md",
        "zed/extension.toml",
        "zed/languages/art-vue/brackets.scm",
        "zed/languages/art-vue/config.toml",
        "zed/languages/art-vue/highlights.scm",
        "zed/languages/art-vue/indents.scm",
        "zed/languages/art-vue/injections.scm",
        "zed/languages/art-vue/outline.scm",
        "zed/languages/art-vue/overrides.scm",
        "zed/src/lib.rs",
    ];
    editor_archive::require_entries(
        &archive,
        &entries,
        &required,
        editor_archive::read_tar_text,
        "Zed archive",
    )?;
    assert_allowed(
        &entries,
        &[
            "zed/",
            "zed/Cargo.lock",
            "zed/Cargo.toml",
            "zed/LICENSE",
            "zed/README.md",
            "zed/extension.toml",
            "zed/languages/",
            "zed/languages/art-vue/",
            "zed/languages/art-vue/brackets.scm",
            "zed/languages/art-vue/config.toml",
            "zed/languages/art-vue/highlights.scm",
            "zed/languages/art-vue/indents.scm",
            "zed/languages/art-vue/injections.scm",
            "zed/languages/art-vue/outline.scm",
            "zed/languages/art-vue/overrides.scm",
            "zed/src/",
            "zed/src/lib.rs",
        ],
        "Zed archive",
    )?;
    assert_no_forbidden(
        &entries,
        "Zed archive",
        &[
            ".git",
            ".github/",
            ".zed/",
            "node_modules/",
            "target/",
            ".DS_Store",
            "~",
        ],
    )?;
    let workspace_version = editor_archive::workspace_version(&root)?;
    let extension = editor_archive::read_tar_text(&archive, "zed/extension.toml")?;
    let cargo = editor_archive::read_tar_text(&archive, "zed/Cargo.toml")?;
    let lib = editor_archive::read_tar_text(&archive, "zed/src/lib.rs")?;
    let art_config = editor_archive::read_tar_text(&archive, "zed/languages/art-vue/config.toml")?;
    let injections =
        editor_archive::read_tar_text(&archive, "zed/languages/art-vue/injections.scm")?;
    let highlights =
        editor_archive::read_tar_text(&archive, "zed/languages/art-vue/highlights.scm")?;
    for (source, needle) in [
        (&extension, r#"id = "vize""#),
        (&extension, r#"name = "Vize""#),
        (&extension, &format!(r#"version = "{workspace_version}""#)),
        (
            &extension,
            r#"repository = "https://github.com/ubugeeei-prod/vize""#,
        ),
        (&extension, "[language_servers.vize]"),
        (&extension, r#"languages = ["Vue", "Art Vue"]"#),
        (&extension, "[language_servers.vize.language_ids]"),
        (&extension, r#""Vue" = "vue""#),
        (&extension, r#""Art Vue" = "art-vue""#),
        (&extension, "[grammars.vue]"),
        (&cargo, r#"name = "vize-zed-extension""#),
        (&cargo, &format!(r#"version = "{workspace_version}""#)),
        (&cargo, r#"edition = "2024""#),
        (&cargo, r#"license = "MIT""#),
        (&cargo, "publish = false"),
        (&cargo, r#"crate-type = ["cdylib"]"#),
        (&cargo, r#"zed_extension_api = "=0.7.0""#),
        (&lib, r#"const SERVER_NAME: &'static str = "vize";"#),
        (&lib, r#"const SERVER_BINARY: &'static str = "vize";"#),
        (&lib, "worktree.which(Self::SERVER_BINARY)"),
        (&lib, r#"unwrap_or_else(|| vec!["lsp".to_string()])"#),
        (&lib, "language_server_initialization_options"),
        (&lib, "recommended_initialization_options"),
        (&lib, r#""editor": true"#),
        (&lib, r#""ecosystem": true"#),
        (&lib, r#""lint": true"#),
        (&lib, r#""typecheck": true"#),
        (&lib, "language_server_workspace_configuration"),
        (&lib, "zed::register_extension!(VizeExtension);"),
        (&art_config, r#"name = "Art Vue""#),
        (&art_config, r#"grammar = "vue""#),
        (&art_config, r#"path_suffixes = ["art.vue"]"#),
        (&art_config, r#"prettier_parser_name = "vue""#),
        (&injections, "directive_attribute"),
        (&injections, "style_element"),
        (&injections, "template_element"),
        (&highlights, "@tag.component.type.constructor"),
    ] {
        editor_archive::expect_contains(source, needle, &format!("Zed package missing {needle}"))?;
    }
    if extension.contains("[grammars.") && extension.contains("[grammars.art-vue]") {
        return Err("Zed extension.toml must not declare dashed grammar ids".to_string());
    }
    println!(
        "Zed package smoke passed: {} ({} entries)",
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
