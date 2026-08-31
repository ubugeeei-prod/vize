#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! glob = "0.3"
//! regex = "1"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! sha1 = "0.10"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../../rust/common.rs"]
mod common;

use regex::Regex;
use sha1::{Digest, Sha1};
use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

const VUE_GLOBS: &[&str] = &[
    "tests/fixtures/**/*.vue",
    "tests/_fixtures/_projects/**/*.vue",
    "playground/src/**/*.vue",
    "playground/e2e/**/*.vue",
];

const FOLIO_GLOBS: &[&str] = &[
    "crates/vize_davinci/tests/fixtures/**/*.folio",
    "crates/vize_s2/tests/fixtures/**/*.folio",
];

fn main() -> ExitCode {
    common::main_result(run())
}

fn run() -> Result<(), String> {
    let root = common::repo_root()?;
    let corpus_root = root.join("tests/fuzz/corpus");

    let sfc_dir = reset_corpus(&corpus_root, "sfc_parse")?;
    let template_lexer_dir = reset_corpus(&corpus_root, "template_lexer")?;
    let expression_dir = reset_corpus(&corpus_root, "js_ts_expression")?;
    let css_dir = reset_corpus(&corpus_root, "css_parse")?;
    let template_dir = reset_corpus(&corpus_root, "template_compile")?;
    let s1_lowering_dir = reset_corpus(&corpus_root, "s1_lowering")?;
    let folio_dir = reset_corpus(&corpus_root, "folio_parse")?;

    let mut folio_count = 0usize;
    for file in glob_files(&root, FOLIO_GLOBS)? {
        write_seed(
            &folio_dir,
            &fs::read(&file).map_err(|error| format!("cannot read {}: {error}", file.display()))?,
        )?;
        folio_count += 1;
    }

    let vue_files = glob_files(&root, VUE_GLOBS)?;
    let script_block = Regex::new(r"(?is)<script\b[^>]*>(.*?)</script>").unwrap();
    let style_block = Regex::new(r"(?is)<style\b[^>]*>(.*?)</style>").unwrap();
    let interpolation = Regex::new(r"(?s)\{\{(.*?)\}\}").unwrap();

    let mut sfc_count = 0usize;
    let mut template_count = 0usize;
    let mut expression_count = 0usize;
    let mut style_count = 0usize;

    for file in &vue_files {
        let content = fs::read_to_string(file)
            .map_err(|error| format!("cannot read {}: {error}", file.display()))?;
        write_seed(&sfc_dir, content.as_bytes())?;
        sfc_count += 1;

        if let Some(template) = extract_template_block(&content) {
            write_seed(&template_lexer_dir, template.as_bytes())?;
            write_seed(&template_dir, template.as_bytes())?;
            write_seed(&s1_lowering_dir, template.as_bytes())?;
            for capture in interpolation.captures_iter(template) {
                write_seed(&expression_dir, capture[1].trim().as_bytes())?;
                expression_count += 1;
            }
            template_count += 1;
        }

        for capture in script_block.captures_iter(&content) {
            write_seed(&expression_dir, capture[1].as_bytes())?;
            expression_count += 1;
        }

        for capture in style_block.captures_iter(&content) {
            write_seed(&css_dir, capture[1].as_bytes())?;
            style_count += 1;
        }
    }

    println!(
        "Seeded {sfc_count} sfc_parse entries, {template_count} template entries (template_lexer/template_compile/s1_lowering), {expression_count} JS/TS expression entries, {style_count} CSS entries, and {folio_count} folio pages from {} fixtures.",
        vue_files.len() + folio_count
    );
    Ok(())
}

fn glob_files(root: &Path, patterns: &[&str]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for pattern in patterns {
        let absolute = root.join(pattern);
        let pattern = absolute
            .to_str()
            .ok_or_else(|| format!("non-utf8 glob pattern: {}", absolute.display()))?;
        for entry in glob::glob(pattern).map_err(|error| error.to_string())? {
            files.push(entry.map_err(|error| error.to_string())?);
        }
    }
    files.sort();
    Ok(files)
}

fn reset_corpus(root: &Path, target: &str) -> Result<PathBuf, String> {
    let dir = root.join(target);
    if dir.exists() {
        fs::remove_dir_all(&dir)
            .map_err(|error| format!("cannot reset {}: {error}", dir.display()))?;
    }
    fs::create_dir_all(&dir)
        .map_err(|error| format!("cannot create {}: {error}", dir.display()))?;
    Ok(dir)
}

fn write_seed(dir: &Path, content: &[u8]) -> Result<(), String> {
    if content.is_empty() {
        return Ok(());
    }
    fs::write(dir.join(hash(content)), content)
        .map_err(|error| format!("cannot write seed in {}: {error}", dir.display()))
}

fn hash(content: &[u8]) -> String {
    let digest = Sha1::digest(content);
    format!("{digest:x}").chars().take(16).collect()
}

fn extract_template_block(source: &str) -> Option<&str> {
    let lower = source.to_ascii_lowercase();
    let open_start = lower.find("<template")?;
    let after_name = open_start + "<template".len();
    let open_end = source[after_name..].find('>')? + after_name + 1;
    let close = lower[open_end..].find("</template>")? + open_end;
    Some(&source[open_end..close])
}
