#![allow(clippy::disallowed_types)] // Fixture files arrive through `std::fs` as std strings.
//! Davinci P4-12c corpus-runnable entry for the pug dialect — the P2-8
//! lane shape (`davinci_lowering_corpus.rs`) for `<template lang="pug">`,
//! and the corpus half of the P4-12c compile oracle.
//!
//! Compiled only with the `davinci-differential` feature. Pins the
//! committed fixture census, then, with
//! `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>`, sweeps every `.vue` file
//! under `<dir>` whose inline template is pug:
//!
//! - **TS-19**: the pug S1 render is the authored template body;
//! - **TS-20**: `lower_pug` is total, the Vue lowering's soundness oracle
//!   holds on the derived template, every diagnostic sits in the pug;
//! - **oracle**: the derived template's sha256 equals the committed
//!   baseline's hash of the pinned `pug@3.0.4` rendering
//!   (`tests/_fixtures/davinci-pug/corpus-baseline.tsv`, pinned by
//!   `tests/tooling/davinci-pug-corpus-oracle.test.ts`), and the SFC
//!   compiles byte-identically to its twin whose template is that HTML in
//!   the DOM, SSR and Vapor lanes — the compile oracle;
//! - **scope proof**: every baseline row of a project present under
//!   `<dir>` is visited, and every pug SFC visited has a row.
//!
//! ```text
//! VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests/_fixtures/_git \
//!     cargo test -p vize_s1_to_s2 --features davinci-differential \
//!     --test davinci_pug_corpus -- --nocapture
//! ```

mod support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use support::assert_sound;
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_s0::Allocator;
use vize_s1::pug::{check_pug_fidelity, parse_pug};
use vize_s1_to_s2::lower::pug::lower_pug;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/davinci-pug")
}

fn fixture_count(dir: &str) -> usize {
    std::fs::read_dir(fixture_root().join(dir))
        .expect("fixture directory")
        .filter(|entry| {
            entry
                .as_ref()
                .is_ok_and(|entry| entry.path().extension().is_some_and(|ext| ext == "pug"))
        })
        .count()
}

/// `(project, file) → sha256` from the committed corpus baseline.
fn baseline() -> BTreeMap<(String, String), String> {
    let text = std::fs::read_to_string(fixture_root().join("corpus-baseline.tsv"))
        .expect("corpus baseline");
    text.lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 4, "baseline row: {line}");
            ((fields[0].into(), fields[2].into()), fields[3].into())
        })
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut out, byte| {
        use std::fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
        out
    })
}

/// The SFC with its pug template body replaced by `html` and the `lang`
/// attribute dropped: the oracle's "same SFC, template replaced".
fn html_twin(source: &str, body: (usize, usize), tag_start: usize, html: &str) -> String {
    let head = &source[tag_start..body.0];
    let lang = ["lang=\"pug\"", "lang='pug'", "lang=pug"]
        .iter()
        .find_map(|spelling| head.find(spelling).map(|at| (at, spelling.len())))
        .expect("the opening tag names lang pug");
    let lang_start = tag_start + head[..lang.0].trim_end().len();
    [
        &source[..lang_start],
        &source[tag_start + lang.0 + lang.1..body.0],
        html,
        &source[body.1..],
    ]
    .concat()
}

fn compile_all(source: &str) -> Vec<Result<String, String>> {
    let mut ssr = SfcCompileOptions::default();
    ssr.template.ssr = true;
    let vapor = SfcCompileOptions {
        vapor: true,
        ..SfcCompileOptions::default()
    };
    [SfcCompileOptions::default(), ssr, vapor]
        .into_iter()
        .map(|options| {
            let descriptor = parse_sfc(source, SfcParseOptions::default())
                .map_err(|error| String::from(error.message.as_str()))?;
            compile_sfc(&descriptor, options)
                .map(|result| String::from(result.code.as_str()))
                .map_err(|error| String::from(error.message.as_str()))
        })
        .collect()
}

/// TS-19, TS-20 and the oracle on one pug SFC: how many of the three
/// lanes compiled (identically, twin and pug), or `None` when refused.
fn check(source: &str, expected: &str, context: &str) -> Option<usize> {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parsed before");
    let template = descriptor.template.as_ref().expect("pug template");
    let content = template.content.as_ref();
    let allocator = Allocator::new();
    let (tree, errors) = parse_pug(&allocator, content);
    assert_eq!(check_pug_fidelity(&tree), Ok(()), "TS-19: {context}");
    let lowered = lower_pug(&allocator, &tree, &errors);
    for diagnostic in &lowered.diagnostics {
        assert!(diagnostic.span.end as usize <= content.len(), "{context}");
    }
    assert_sound(lowered.html, context);
    if lowered.template.has_errors() {
        return None;
    }
    let digest = hex(&Sha256::digest(lowered.html.as_bytes()));
    assert_eq!(
        digest, expected,
        "{context}: derived template != pinned pug"
    );
    let body = (template.loc.start, template.loc.end);
    let twin = html_twin(source, body, template.loc.tag_start, lowered.html);
    let lanes = compile_all(source);
    assert_eq!(lanes, compile_all(&twin), "{context}: compile oracle");
    Some(lanes.iter().filter(|lane| lane.is_ok()).count())
}

#[test]
fn pug_corpus_matches_the_pinned_pug_and_compiles_like_it() {
    assert_eq!(
        (fixture_count("matrix"), fixture_count("refused")),
        (26, 14)
    );
    let rows = baseline();
    assert_eq!(rows.len(), 498, "the corpus baseline scope is pinned");
    let Some(sweep) = davinci_test_support::corpus::resolve_env_sweep() else {
        eprintln!("VIZE_DAVINCI_DIFFERENTIAL_CORPUS unset: committed fixtures only");
        return;
    };
    let mut visited = BTreeMap::new();
    let (mut refused, mut compiled) = (0u64, 0usize);
    for file in &sweep.files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        let Ok(descriptor) = parse_sfc(&source, SfcParseOptions::default()) else {
            continue;
        };
        let Some(template) = descriptor.template.as_ref() else {
            continue;
        };
        if template.src.is_some() || template.lang.as_deref() != Some("pug") {
            continue;
        }
        let relative = file.strip_prefix(&sweep.root).expect("under the root");
        let mut parts = relative
            .iter()
            .map(|part| part.to_string_lossy().into_owned());
        let project = parts.next().expect("a project directory");
        let path = parts.collect::<Vec<_>>().join("/");
        let key = (project, path);
        let expected = rows
            .get(&key)
            .unwrap_or_else(|| panic!("{key:?}: pug SFC missing from the corpus baseline"));
        match check(&source, expected, &file.to_string_lossy()) {
            Some(lanes) => compiled += lanes,
            None => refused += 1,
        }
        visited.insert(key, ());
    }
    let present = |project: &str| Path::new(&sweep.root).join(project).is_dir();
    for key in rows.keys().filter(|(project, _)| present(project)) {
        assert!(
            visited.contains_key(key),
            "{key:?}: baseline row not found in the corpus"
        );
    }
    eprintln!(
        "davinci pug corpus sweep: files={} pug={} refused={refused} compiled_lanes={compiled}",
        sweep.files.len(),
        visited.len()
    );
    assert_eq!(refused, 0, "every corpus pug template is static pug");
}
