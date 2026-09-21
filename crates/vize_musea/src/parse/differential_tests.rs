//! P4-13 differential lane: the S0/S1 parser against the retained hand
//! scanner (`legacy`), over every committed musea fixture.
//!
//! The oracle is exact equality of a full fingerprint per file: the
//! `Debug` rendering of the whole parse result, the file-absolute offset of
//! every borrowed slice (LSP consumers do pointer arithmetic on them), and
//! every downstream artifact built from the descriptor (Storybook CSF, Vue,
//! component docs, catalog entry, palette, owned JSON) plus the status
//! warnings. A fixture on the intentional-divergence ledger must *differ*
//! and match its pinned expectation in `divergence_tests.rs` instead.
//!
//! Scope proof: the committed corpus count is pinned, so an empty or
//! half-read fixture directory fails. The committed files are `*.vue.txt`
//! (many are malformed by design, so they stay out of the repository's Vue
//! lint and format lanes). `VIZE_MUSEA_DIFFERENTIAL_CORPUS=<dir>`
//! widens the lane with every `*.vue` file under `<dir>` (used for the
//! generated 240-file Musea benchmark corpus; counts are reported, not
//! pinned).

use std::path::{Path, PathBuf};

use super::legacy::{parse_art_legacy, parse_art_status_warnings_legacy};
use super::{art_status_warnings, parse_art};
use crate::docs::{CatalogEntry, DocOptions, generate_component_doc};
use crate::palette::{PaletteOptions, generate_palette};
use crate::types::{ArtParseOptions, ArtParseResult};
use crate::{transform_to_csf, transform_to_vue};
use vize_s0::{Allocator, String, append};

/// Committed corpus size (`tests/fixtures/differential`, excluding the
/// manifest). Changing the corpus means changing this number.
const COMMITTED_FIXTURES: usize = 217;

/// Fixtures where the new parser is intentionally not bug-compatible; each
/// class has an exact expectation in `divergence_tests.rs` (D1–D4).
const INTENTIONAL_DIVERGENCES: &[&str] = &[
    // D1: `<variant … />` is an empty variant.
    "content-musea-and-css-1e05f9370f2a.art.vue.txt",
    "content-musea-and-css-8d88db42666b.art.vue.txt",
    "content-musea-and-css-9e7833924193.art.vue.txt",
    "content-musea-and-css-ddb6999a589e.art.vue.txt",
    "content-musea-and-css-e208bf92b367.art.vue.txt",
    // D3: `<style>` inside `<art>` is not an SFC style block.
    "vize-maestro-configured-lint-tests-8f44edc047c4.art.vue.txt",
    // D4: an unsplittable container reports the container error.
    "vize-patina-html-66050b9db3b6.art.vue.txt",
    // D2: `title = "…"` (spaced `=`) is an attribute value.
    "vize-patina-require-title-45a52dcf3b55.art.vue.txt",
];

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/differential")
}

/// Vue sources: `*.vue`, or `*.vue.txt` for the committed corpus — stored as
/// text because many fixtures are malformed by design and must stay out of
/// the repository's Vue lint and format lanes.
fn is_vue_source(path: &Path) -> bool {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let stem = name.strip_suffix(".txt").unwrap_or(&name);
    Path::new(stem).extension().is_some_and(|ext| ext == "vue")
}

fn collect_vue_files(dir: &Path, out: &mut std::vec::Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).expect("readable corpus directory");
    for entry in entries {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            collect_vue_files(&path, out);
        } else if is_vue_source(&path) {
            out.push(path);
        }
    }
}

fn corpus(dir: &Path) -> std::vec::Vec<(String, std::vec::Vec<u8>)> {
    let mut paths = std::vec::Vec::new();
    collect_vue_files(dir, &mut paths);
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let name = path
                .strip_prefix(dir)
                .expect("corpus path under its root")
                .to_string_lossy();
            let bytes = std::fs::read(&path).expect("readable fixture");
            (String::from(name.as_ref()), bytes)
        })
        .collect()
}

/// File-absolute offset of `slice` when it borrows `source`, or `arena`.
fn place(out: &mut String, label: &str, slice: &str, source: &str) {
    let base = source.as_ptr() as usize;
    let ptr = slice.as_ptr() as usize;
    if ptr >= base && ptr + slice.len() <= base + source.len() {
        append!(*out, "{label}@{}+{}\n", ptr - base, slice.len());
    } else {
        append!(*out, "{label}@arena\n");
    }
}

fn fingerprint(result: &ArtParseResult<'_>, source: &str, filename: &str) -> String {
    let mut out = String::default();
    append!(out, "{result:#?}\n");
    let Ok(desc) = result else {
        return out;
    };
    let meta = &desc.metadata;
    place(&mut out, "title", meta.title, source);
    for (label, value) in [
        ("description", meta.description),
        ("component", meta.component),
        ("category", meta.category),
    ] {
        if let Some(value) = value {
            place(&mut out, label, value, source);
        }
    }
    for tag in meta.tags.iter() {
        place(&mut out, "tag", tag, source);
    }
    for variant in &desc.variants {
        place(&mut out, "variant.name", variant.name, source);
        place(&mut out, "variant.template", variant.template, source);
    }
    for script in [&desc.script_setup, &desc.script].into_iter().flatten() {
        place(&mut out, "script.content", script.content, source);
        if let Some(lang) = script.lang {
            place(&mut out, "script.lang", lang, source);
        }
    }
    for style in desc.styles.iter() {
        place(&mut out, "style.content", style.content, source);
        if let Some(lang) = style.lang {
            place(&mut out, "style.lang", lang, source);
        }
    }
    let docs = DocOptions {
        include_source: true,
        include_templates: true,
        include_metadata: true,
        include_toc: true,
        toc_threshold: 1,
        ..DocOptions::default()
    };
    append!(out, "csf: {:#?}\n", transform_to_csf(desc));
    append!(out, "vue: {:#?}\n", transform_to_vue(desc));
    append!(out, "doc: {:#?}\n", generate_component_doc(desc, &docs));
    append!(
        out,
        "catalog: {:#?}\n",
        CatalogEntry::from_descriptor(desc, "/gallery")
    );
    append!(
        out,
        "palette: {:#?}\n",
        generate_palette(desc, &PaletteOptions::default())
    );
    append!(out, "filename: {filename}\n");
    out
}

/// Parse `source` both ways and return (legacy, new) fingerprints.
fn both_ways(name: &str, source: &str) -> (String, String) {
    let options = || ArtParseOptions {
        filename: String::from(name),
    };
    let allocator = Allocator::new();
    let old = parse_art_legacy(&allocator, source, options());
    let new = parse_art(&allocator, source, options());
    let mut old_print = fingerprint(&old, source, name);
    let mut new_print = fingerprint(&new, source, name);
    let owned = |result: ArtParseResult<'_>| {
        result
            .map(|desc| serde_json::to_string(&desc.into_owned()).expect("owned JSON"))
            .unwrap_or_default()
    };
    append!(old_print, "owned: {}\n", owned(old));
    append!(new_print, "owned: {}\n", owned(new));
    append!(
        old_print,
        "warnings: {:?}\n",
        parse_art_status_warnings_legacy(&allocator, source, name).as_slice()
    );
    append!(
        new_print,
        "warnings: {:?}\n",
        art_status_warnings(&allocator, source, name).as_slice()
    );
    (old_print, new_print)
}

/// The first differing line pair, for the failure message.
fn first_difference(old: &str, new: &str) -> String {
    let mut out = String::default();
    for (line, (a, b)) in old.lines().zip(new.lines()).enumerate() {
        if a != b {
            append!(out, "line {}: legacy `{a}` vs s0/s1 `{b}`", line + 1);
            return out;
        }
    }
    append!(
        out,
        "length: legacy {} lines vs s0/s1 {} lines",
        old.lines().count(),
        new.lines().count()
    );
    out
}

/// Run the lane over `dir`; returns (files, divergent fixture names) and
/// logs each divergence's first differing line.
fn run_lane(dir: &Path) -> (usize, std::vec::Vec<String>) {
    let files = corpus(dir);
    let mut divergent = std::vec::Vec::new();
    for (name, bytes) in &files {
        let Ok(source) = core::str::from_utf8(bytes) else {
            continue;
        };
        let (old, new) = both_ways(name, source);
        if old != new {
            std::eprintln!("divergent {name}: {}", first_difference(&old, &new));
            divergent.push(name.clone());
        }
    }
    (files.len(), divergent)
}

#[test]
fn committed_musea_fixtures_agree_with_the_hand_scanner() {
    let (files, divergent) = run_lane(&fixture_dir());
    assert_eq!(files, COMMITTED_FIXTURES, "scope proof: corpus size");
    let divergent: std::vec::Vec<&str> = divergent.iter().map(String::as_str).collect();
    assert_eq!(divergent, INTENTIONAL_DIVERGENCES);
}

/// The in-repo gallery (`examples/vite-musea`, inline `<art>` blocks in
/// real components) is the real-project art surface: it admits no ledger.
#[test]
fn example_gallery_agrees_with_the_hand_scanner_exactly() {
    let gallery = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/vite-musea/src");
    let (files, divergent) = run_lane(&gallery);
    assert_eq!(files, 7, "scope proof: App.vue + six gallery components");
    assert_eq!(divergent, std::vec::Vec::<String>::new());
}

#[test]
fn widened_corpus_agrees_with_the_hand_scanner() {
    let Some(dir) = std::env::var_os("VIZE_MUSEA_DIFFERENTIAL_CORPUS") else {
        return;
    };
    let (files, divergent) = run_lane(Path::new(&dir));
    std::eprintln!(
        "musea differential (widened): {files} files, {} divergent",
        divergent.len()
    );
    assert_ne!(files, 0, "scope proof: widened corpus is empty");
    assert_eq!(divergent, std::vec::Vec::<String>::new());
}

/// Print one fixture's old/new fingerprints (triage aid for a failing run):
/// `VIZE_MUSEA_DIFFERENTIAL_SHOW=<fixture> cargo test -p vize_musea show_`.
#[test]
fn show_fixture_fingerprints() {
    let Some(name) = std::env::var_os("VIZE_MUSEA_DIFFERENTIAL_SHOW") else {
        return;
    };
    let path = fixture_dir().join(&name);
    let source = std::fs::read_to_string(&path).expect("fixture");
    let (old, new) = both_ways(&name.to_string_lossy(), &source);
    std::eprintln!("--- legacy\n{old}\n--- s0/s1\n{new}");
}
