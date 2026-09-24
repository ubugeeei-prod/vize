//! The in-tree Vue dialect crosses the input-dialect contract losslessly.
//!
//! The first-party tier answers through the same session and acceptance as
//! an external guest. Over the TS-19 battery (and every golden case) the
//! serialized answer carries the in-tree boundary exactly: the S1 page
//! rebuilds the parsed tree, the S2 page is the lowered op tree's folio,
//! the diagnostics convert back to the in-tree channel unchanged, and the
//! out-of-process wire's JSON round-trips every value. The committed
//! goldens that the TS-48 echo guest replays are pinned to this output.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::string_slice,
    reason = "tests assert by panicking"
)]

mod support;

use davinci_test_support::surface_fixture::{MALFORMED, WELL_FORMED};
use vize_extension_host::vue::VueDialect;
use vize_extension_host::{LoweredBlock, Session, SourceBlock, SurfacePage};
use vize_s0::{Allocator, SourceRoot, String};
use vize_s1_to_s2::lower_source_block;
use vize_s2::folio::S2Folio;

use support::{BLESS_ENV, CASES, golden_texts};

fn session() -> Session<VueDialect> {
    Session::open(VueDialect::default()).expect("the Vue dialect negotiates")
}

#[test]
fn the_vue_dialect_negotiates_its_lang_and_pages() {
    let session = session();
    let negotiated = session.negotiated();
    assert_eq!(negotiated.protocol_version, 1);
    assert_eq!(negotiated.langs, [String::from("html")]);
    assert_eq!(negotiated.ignored, Vec::<String>::new());
}

/// Lower `source` in-tree, directly, at `base`: the oracle the contract
/// answer is compared against.
fn assert_lossless(source: &str, base: u32, context: &str) {
    let block = SourceBlock {
        source: String::from(source),
        base,
        lang: None,
    };
    let accepted = session()
        .lower_block(&block)
        .unwrap_or_else(|error| panic!("{context}: {error}"));

    let mut root_text = String::default();
    root_text.extend(core::iter::repeat_n('\u{20}', base as usize));
    root_text.push_str(source);
    let root = SourceRoot::new(&root_text).expect("small root");
    let slice = &root_text[base as usize..];
    let frame = root.block(slice, base).expect("own slice");
    let allocator = Allocator::new();
    let (tree, errors) = vize_s1::parse(&allocator, slice);
    let lowered = lower_source_block(&allocator, &tree, &errors, frame);

    assert_eq!(
        accepted.surface,
        SurfacePage::of(&tree),
        "s1 page: {context}"
    );
    let rebuilt_arena = Allocator::new();
    let rebuilt = accepted
        .surface
        .materialize(&rebuilt_arena, &block.source)
        .expect("accepted pages tile");
    assert_eq!(
        SurfacePage::of(&rebuilt),
        accepted.surface,
        "rebuilt tree: {context}"
    );
    assert_eq!(
        accepted.semantic,
        S2Folio::of(&lowered.root.ops),
        "s2 page: {context}"
    );
    assert_eq!(
        accepted.diagnostics, lowered.diagnostics,
        "diagnostics: {context}"
    );

    let wire = serde_json::to_vec(&accepted.lowered).expect("serializes");
    let back: LoweredBlock = serde_json::from_slice(&wire).expect("deserializes");
    assert_eq!(back, accepted.lowered, "wire round trip: {context}");
}

#[test]
fn the_battery_crosses_the_contract_losslessly_at_any_base() {
    let mut checked = 0usize;
    for fixture in WELL_FORMED.iter().chain(MALFORMED) {
        for base in [0, 7, 4_096] {
            assert_lossless(fixture.source, base, fixture.name);
            checked += 1;
        }
    }
    assert_eq!(checked, 126);
}

#[test]
fn committed_goldens_are_the_in_tree_vue_output() {
    let bless = std::env::var_os(BLESS_ENV).is_some();
    let mut session = session();
    for case in CASES {
        let block = case.block();
        let accepted = session
            .lower_block(&block)
            .unwrap_or_else(|error| panic!("{}: {error}", case.name));
        let produced = golden_texts(&accepted.lowered);
        if bless {
            for (ext, text) in ["s1.folio", "s2.folio", "diagnostics"]
                .iter()
                .zip(&produced)
            {
                let path = support::golden_dir().join(case.file(ext).as_str());
                std::fs::write(&path, text.as_bytes()).expect("writes the golden");
            }
            continue;
        }
        assert_eq!(
            case.committed(),
            produced,
            "{}: stale golden; rerun with {BLESS_ENV}=1 and review the diff",
            case.name
        );
    }
    assert_eq!(CASES.len(), 4);
}
