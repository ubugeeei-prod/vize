//! TS-18: the invalid-folio fixture set (P2-6).
//!
//! `tests/fixtures/invalid/` holds hand-built S2 pages that are
//! grammar-valid — `parse` accepts them, which is the point: the verifier
//! owns what the grammar deliberately does not — and semantically invalid.
//! Each `.folio` is committed beside its exact expected rendering
//! (`.expected`: one `{code} @{start}:{end} {message}` line per violation,
//! page order, canonical `en` locale), and the harness compares **whole
//! files** — no partial matching (TS-13). The fixture count is pinned so a
//! fixture that stops being discovered fails loudly instead of silently
//! shrinking the suite.

use std::path::PathBuf;

use vize_davinci::folio::Folio;
use vize_s0::cstr;
use vize_s2::folio::DisegnoFolio;
use vize_s2::verify::{Rigor, verify};

/// Every committed invalid page. Grows only deliberately.
const INVALID_FIXTURES: usize = 15;

/// The invalid pages whose only violations are canonical-form ones
/// (S2V004–S2V006): structurally sound, so [`Rigor::Raw`] must accept
/// them — the rigor split is load-bearing, not decorative.
const CANONICAL_ONLY: &[&str] = &[
    "empty-if.folio",
    "floating-else.folio",
    "leading-else.folio",
    "else-mid-chain.folio",
];

fn invalid_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("invalid")
}

/// The `.folio` files of the invalid set, sorted by file name.
fn invalid_pages() -> Vec<PathBuf> {
    let mut pages: Vec<PathBuf> = std::fs::read_dir(invalid_dir())
        .expect("the committed invalid fixture directory reads")
        .map(|entry| entry.expect("fixture directory entries read").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "folio")
        })
        .collect();
    pages.sort();
    pages
}

/// Render `violations` the way the `.expected` files are committed: one
/// line per violation, LF-terminated.
fn render(violations: &[vize_s2::verify::Violation]) -> vize_s0::String {
    let mut rendered = vize_s0::String::default();
    for violation in violations {
        rendered.push_str(cstr!("{violation}\n").as_str());
    }
    rendered
}

#[test]
fn every_invalid_page_is_rejected_with_its_exact_committed_diagnostic() {
    let pages = invalid_pages();
    assert_eq!(pages.len(), INVALID_FIXTURES);
    for page in pages {
        let name = page
            .file_name()
            .expect("fixture paths carry file names")
            .to_str()
            .expect("fixture names are UTF-8");
        let text = std::fs::read_to_string(&page).expect("committed fixture reads");
        let folio = DisegnoFolio::parse(&text)
            .unwrap_or_else(|error| panic!("{name} must be grammar-valid: {error:?}"));
        let rendered = render(&verify(&folio, Rigor::Canonical));
        let expected = std::fs::read_to_string(page.with_extension("expected"))
            .expect("every invalid page has a committed .expected twin");
        assert_eq!(rendered.as_str(), expected.as_str(), "{name}");
    }
}

/// The fixtures are canonical text, so drift is loud: a page that stops
/// round-tripping byte-identically has been edited into a non-canonical
/// spelling and must be re-normalized, not silently reinterpreted.
#[test]
fn every_invalid_page_is_committed_in_canonical_spelling() {
    for page in invalid_pages() {
        let name = page
            .file_name()
            .expect("fixture paths carry file names")
            .to_str()
            .expect("fixture names are UTF-8");
        let text = std::fs::read_to_string(&page).expect("committed fixture reads");
        let folio = DisegnoFolio::parse(&text).expect("grammar-valid fixture parses");
        let mut printed = vize_s0::String::default();
        folio
            .print(&mut printed, vize_davinci::folio::FolioMode::Full)
            .expect("printing into a string cannot fail");
        assert_eq!(printed.as_str(), text.as_str(), "{name}");
    }
}

#[test]
fn canonical_only_pages_hold_every_structural_invariant_at_raw_rigor() {
    let dir = invalid_dir();
    for name in CANONICAL_ONLY {
        let text = std::fs::read_to_string(dir.join(name)).expect("committed fixture reads");
        let folio = DisegnoFolio::parse(&text).expect("grammar-valid fixture parses");
        assert_eq!(verify(&folio, Rigor::Raw), vec![], "{name}");
    }
}

#[test]
fn the_committed_reference_page_verifies_clean_at_both_rigors() {
    let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("reference.folio");
    let text = std::fs::read_to_string(reference).expect("committed reference page reads");
    let folio = DisegnoFolio::parse(&text).expect("the reference page parses");
    assert_eq!(verify(&folio, Rigor::Raw), vec![]);
    assert_eq!(verify(&folio, Rigor::Canonical), vec![]);
}
