//! Davinci P2-7 TS-19 lane — corpus-runnable entry.
//!
//! Compiled only with the `davinci-differential` feature (see `[[test]]
//! required-features` in Cargo.toml), the P1-6/P1-7 lane shape. Runs the
//! committed battery with its exact-pinned scope and hole census (the
//! cfg-regression witness — the plain suite pins the same numbers, so a
//! feature-wiring regression fails loudly in both lanes), then, with
//! `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>`, additionally sweeps every
//! `.vue` file under `<dir>` (e.g. `tests/_fixtures/_git` for the
//! hydrated ecosystem corpus): the **whole file's bytes** go through the
//! S1 parser — SFC block markup as markup, `<script>`/`<style>` content
//! as raw text — and `render(parse(src)) == src` is asserted per file.
//!
//! Run:
//!
//! ```text
//! VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests/_fixtures/_git \
//!     cargo test -p vize_sinopia --features davinci-differential \
//!     --test davinci_surface_corpus -- --nocapture
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use davinci_test_support::surface_fixture as common;
use vize_carton::{Allocator, String};
use vize_sinopia::{HoleCounts, hole_counts, parse, render};

fn collect_vue_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut children: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    children.sort();
    for child in children {
        if child.is_dir() {
            // node_modules trees repeat the same shipped sources; the
            // sweep targets project code.
            if child.file_name().is_some_and(|name| name == "node_modules") {
                continue;
            }
            collect_vue_files(&child, out);
        } else if child.extension().is_some_and(|ext| ext == "vue") {
            out.push(child);
        }
    }
}

fn assert_round_trip(source: &str, context: &str) {
    let allocator = Allocator::new();
    let (tree, _errors) = parse(&allocator, source);
    let mut out = String::default();
    render(&tree, &mut |piece| out.push_str(piece));
    assert_eq!(out.as_str(), source, "S1 fidelity diverged: {context}");
}

#[test]
fn surface_corpus_round_trips() {
    // -- committed battery, exact-pinned scope ---------------------------
    assert_eq!(common::WELL_FORMED.len(), 16);
    assert_eq!(common::MALFORMED.len(), 26);
    for fixture in common::WELL_FORMED.iter().chain(common::MALFORMED) {
        let allocator = Allocator::new();
        let (tree, _errors) = parse(&allocator, fixture.source);
        let mut out = String::default();
        render(&tree, &mut |piece| out.push_str(piece));
        assert_eq!(out.as_str(), fixture.source, "fidelity: {}", fixture.name);
        let expected = HoleCounts {
            missing_tokens: fixture.missing_tokens,
            missing_close_tags: fixture.missing_close_tags,
            unexpected_nodes: fixture.unexpected_nodes,
        };
        assert_eq!(
            hole_counts(&tree),
            expected,
            "hole census: {}",
            fixture.name
        );
    }

    // -- optional corpus sweep -------------------------------------------
    let Some(corpus_root) = std::env::var_os("VIZE_DAVINCI_DIFFERENTIAL_CORPUS") else {
        eprintln!("VIZE_DAVINCI_DIFFERENTIAL_CORPUS unset: committed battery only");
        return;
    };
    let corpus_root = PathBuf::from(corpus_root);
    // Cargo runs test binaries from the package directory; let
    // workspace-root-relative paths (`tests/_fixtures/_git`) work too.
    let corpus_root = if corpus_root.is_relative() && !corpus_root.is_dir() {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(&corpus_root)
    } else {
        corpus_root
    };
    assert!(
        corpus_root.is_dir(),
        "VIZE_DAVINCI_DIFFERENTIAL_CORPUS must name a directory: {}",
        corpus_root.display()
    );
    let mut files = Vec::new();
    collect_vue_files(&corpus_root, &mut files);
    assert!(
        !files.is_empty(),
        "corpus sweep found no .vue files under {}",
        corpus_root.display()
    );
    let mut checked = 0u64;
    for file in &files {
        let Ok(source) = fs::read_to_string(file) else {
            continue;
        };
        let mut context = String::default();
        context.push_str(file.to_string_lossy().as_ref());
        assert_round_trip(&source, context.as_str());
        checked += 1;
    }
    eprintln!(
        "davinci surface corpus sweep: files={} checked={}",
        files.len(),
        checked
    );
}
