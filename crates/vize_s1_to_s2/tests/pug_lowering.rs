#![allow(clippy::disallowed_types)] // Fixture files arrive through `std::fs` as std strings.
//! TS-20 for the pug dialect (Davinci P4-12c): the pug → Vue-template
//! desugaring and the S2 lowering behind it.
//!
//! - **Oracle.** Every matrix fixture derives exactly the committed
//!   `.html`, which `tests/tooling/davinci-pug-oracle.test.ts` proves is
//!   the pinned `pug@3.0.4` rendering; every refused fixture reports
//!   exactly its committed diagnostics (`VIZE_PUG_BLESS=1` rewrites them).
//! - **Totality.** Every fixture and every prefix/suffix truncation lowers
//!   through `lower_pug` with the Vue lowering's soundness laws holding on
//!   the derived template and every diagnostic inside the authored pug.
//! - **Provenance.** Derived-template spans map back onto authored pug.

mod support;

use std::fmt::Write as _;
use std::path::PathBuf;

use support::assert_sound;
use vize_s0::{Allocator, Span};
use vize_s1::pug::parse_pug;
use vize_s1_to_s2::lower::pug::{derive_template_source, lower_pug};

fn fixtures(dir: &str) -> Vec<(String, String, PathBuf)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/davinci-pug")
        .join(dir);
    let mut paths: Vec<_> = std::fs::read_dir(&root)
        .expect("pug fixture directory")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "pug"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            (name, std::fs::read_to_string(&path).expect("fixture"), path)
        })
        .collect()
}

#[test]
fn every_matrix_fixture_derives_the_pinned_pug_html() {
    let matrix = fixtures("matrix");
    assert_eq!(matrix.len(), 26);
    for (name, source, path) in &matrix {
        let expected = std::fs::read_to_string(path.with_extension("html")).expect("html");
        let derived = derive_template_source(source);
        assert!(
            derived.diagnostics.is_empty(),
            "{name}: {:?}",
            derived.diagnostics
        );
        assert_eq!(
            derived.html.as_str(),
            expected,
            "{name}: derived template drifted"
        );
        assert_eq!(
            derived.map.html_len() as usize,
            derived.html.len(),
            "{name}: map coverage"
        );
    }
}

fn render_diagnostics(source: &str) -> String {
    let derived = derive_template_source(source);
    let mut out = String::new();
    for diagnostic in &derived.diagnostics {
        let span = diagnostic.span;
        let text = source
            .get(span.start as usize..span.end as usize)
            .unwrap_or("<oob>");
        let _ = writeln!(
            out,
            "{}..{} {:?} {:?} — {}",
            span.start, span.end, diagnostic.stage, text, diagnostic.message
        );
    }
    out
}

#[test]
fn every_refused_fixture_reports_its_exact_diagnostics() {
    let refused = fixtures("refused");
    assert_eq!(refused.len(), 14);
    let bless = std::env::var("VIZE_PUG_BLESS").is_ok_and(|value| value == "1");
    for (name, source, path) in &refused {
        let actual = render_diagnostics(source);
        assert!(!actual.is_empty(), "{name} must be refused");
        let target = path.with_extension("diagnostics");
        if bless {
            std::fs::write(&target, &actual).expect("bless");
        }
        let expected = std::fs::read_to_string(&target).expect("committed diagnostics");
        assert_eq!(actual, expected, "{name}: diagnostics drifted");
    }
}

/// `lower_pug` on one input: total, sound on the derived template, and
/// every diagnostic inside the authored pug.
fn assert_pug_sound(source: &str, context: &str) {
    let allocator = Allocator::new();
    let (tree, errors) = parse_pug(&allocator, source);
    let lowered = lower_pug(&allocator, &tree, &errors);
    assert_eq!(lowered.html, lowered.template.html.as_str(), "{context}");
    for diagnostic in &lowered.diagnostics {
        let Span { start, end } = diagnostic.span;
        assert!(
            start <= end && end as usize <= source.len(),
            "{context}: {diagnostic:?}"
        );
    }
    assert_sound(lowered.html, context);
}

#[test]
fn lowering_is_total_over_every_fixture_and_truncation() {
    let all: Vec<_> = fixtures("matrix")
        .into_iter()
        .chain(fixtures("refused"))
        .collect();
    for (name, source, _) in &all {
        assert_pug_sound(source, name);
        for (at, _) in source.char_indices() {
            assert_pug_sound(&source[..at], name);
            assert_pug_sound(&source[at..], name);
        }
    }
}

#[test]
fn derived_spans_map_back_to_authored_pug() {
    let source = "div\n  p(title=\"hello\") Hi {{ name }}\n";
    let derived = derive_template_source(source);
    assert_eq!(
        derived.html.as_str(),
        "<div><p title=\"hello\">Hi {{ name }}</p></div>"
    );
    for needle in ["hello", "Hi {{ name }}", "p", "div"] {
        let html_at = derived.html.find(needle).unwrap() as u32;
        let pug_at = source.find(needle).unwrap() as u32;
        let span = derived
            .map
            .to_pug(Span::new(html_at, html_at + needle.len() as u32));
        assert_eq!(
            span,
            Span::new(pug_at, pug_at + needle.len() as u32),
            "{needle}"
        );
    }
}

#[test]
fn vue_lowering_diagnostics_arrive_in_pug_coordinates() {
    let source = "div\n  p(v-else) orphan\n";
    let allocator = Allocator::new();
    let (tree, errors) = parse_pug(&allocator, source);
    let lowered = lower_pug(&allocator, &tree, &errors);
    assert!(lowered.template.diagnostics.is_empty());
    let mapped: Vec<_> = lowered
        .diagnostics
        .iter()
        .map(|d| {
            (
                &source[d.span.start as usize..d.span.end as usize],
                d.message.as_str(),
            )
        })
        .collect();
    assert_eq!(
        mapped,
        [("v-else", "v-else/v-else-if has no adjacent v-if.")],
        "the Vue lowering's diagnostic lands on the authored pug attribute"
    );
}
