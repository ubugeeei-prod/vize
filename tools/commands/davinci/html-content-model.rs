#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! ```
//!
//! Generate the HTML content-model fact table (Davinci P4-11a).
//!
//! `crates/vize_patina/src/html_content_model/whatwg.tsv` is derived from
//! the pinned WHATWG snapshot excerpt in `…/html_content_model/whatwg/`:
//! every row names the spec clause it projects, and every member comes out of
//! the excerpt by a recipe below — no element list is typed by hand.
//!
//! Usage:
//!   rust-script tools/commands/davinci/html-content-model.rs --write
//!   rust-script tools/commands/davinci/html-content-model.rs --check [--table <path>]
//!   rust-script tools/commands/davinci/html-content-model.rs --extract <parsing.html> <indices.html>
//!
//! `--extract` refreshes the excerpt from downloaded multipage spec pages
//! (maintenance only; the snapshot date is recorded in the excerpt).

use std::{env, fs, path::PathBuf, process::ExitCode};

#[path = "../../support/common.rs"]
mod common;
#[path = "../../support/davinci/html_spec.rs"]
mod html_spec;
#[path = "../../support/davinci/html_spec_recipes.rs"]
mod recipes;
#[path = "../../support/davinci/html_spec_sets.rs"]
mod sets;

const TABLE: &str = "crates/vize_patina/src/html_content_model/whatwg.tsv";
const EXCERPT: &str = "crates/vize_patina/src/html_content_model/whatwg";

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<u8, String> {
    let root = common::repo_root()?;
    let args: Vec<String> = env::args().skip(1).collect();
    let excerpt = root.join(EXCERPT);
    match args.first().map(String::as_str) {
        Some("--extract") => {
            let (Some(parsing), Some(indices)) = (args.get(1), args.get(2)) else {
                return Err("--extract <parsing.html> <indices.html>".to_string());
            };
            extract(&excerpt, parsing, indices)?;
            Ok(0)
        }
        Some(mode @ ("--write" | "--check")) => {
            let table = match args.iter().position(|arg| arg == "--table") {
                Some(index) => PathBuf::from(args.get(index + 1).ok_or("--table <path>")?),
                None => root.join(TABLE),
            };
            let generated = generate(&excerpt)?;
            if mode == "--write" {
                fs::write(&table, &generated).map_err(|error| error.to_string())?;
                println!("wrote {}", table.display());
                return Ok(0);
            }
            let committed = fs::read_to_string(&table).unwrap_or_default();
            if committed == generated {
                println!("{} is up to date", table.display());
                return Ok(0);
            }
            println!("stale: {} drifted from the snapshot excerpt", table.display());
            for (index, (left, right)) in committed.lines().zip(generated.lines()).enumerate() {
                if left != right {
                    println!("  first differing line: {}", index + 1);
                    println!("  - {left}");
                    println!("  + {right}");
                    break;
                }
            }
            println!("  Regenerate with: rust-script tools/commands/davinci/html-content-model.rs --write");
            Ok(1)
        }
        _ => Err("usage: html-content-model.rs --write | --check [--table <path>] | --extract <parsing.html> <indices.html>".to_string()),
    }
}

fn extract(excerpt: &PathBuf, parsing: &str, indices: &str) -> Result<(), String> {
    let parsing_html = fs::read_to_string(parsing).map_err(|error| error.to_string())?;
    let indices_html = fs::read_to_string(indices).map_err(|error| error.to_string())?;
    fs::create_dir_all(excerpt).map_err(|error| error.to_string())?;
    let mut text = String::from(recipes::EXCERPT_HEADER);
    for id in html_spec::SECTIONS
        .iter()
        .chain(html_spec::MORE_SECTIONS.iter())
    {
        text.push_str(&format!("## {id}\n"));
        text.push_str(&html_spec::extract_section(&parsing_html, id)?);
        text.push('\n');
    }
    fs::write(excerpt.join("parsing.txt"), text).map_err(|error| error.to_string())?;
    for (file, caption) in [
        ("elements.tsv", "List of elements"),
        ("categories.tsv", "List of element content categories"),
    ] {
        let table = html_spec::extract_table(&indices_html, caption)?;
        let body = format!("{}{table}\n", recipes::EXCERPT_HEADER);
        fs::write(excerpt.join(file), body).map_err(|error| error.to_string())?;
    }
    println!("extracted the snapshot excerpt into {}", excerpt.display());
    Ok(())
}

fn generate(excerpt: &PathBuf) -> Result<String, String> {
    let read = |name: &str| {
        fs::read_to_string(excerpt.join(name)).map_err(|error| format!("{name}: {error}"))
    };
    let spec = html_spec::Spec::new(
        &read("parsing.txt")?,
        &read("elements.tsv")?,
        &read("categories.tsv")?,
    );
    recipes::table(&spec)
}
