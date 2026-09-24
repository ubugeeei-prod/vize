//! The S1 page (`s1-page@1`) laws, exact-equality oracles only:
//!
//! - `parse(print(page)) == page` and `print(parse(text)) == text`;
//! - the page tiles its source, and `materialize` rebuilds a tree whose
//!   render is the source bytes and whose page is the original page;
//!
//! over the TS-19 battery (well-formed and malformed) and every prefix and
//! suffix truncation of it, plus the pinned grammar and its exact refusals.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::string_slice,
    reason = "tests assert by panicking"
)]

use davinci_test_support::surface_fixture::{MALFORMED, WELL_FORMED};
use vize_davinci::folio::{Folio, FolioError};
use vize_extension_host::accept::full_text;
use vize_extension_host::surface_page::{PageNode, PageToken};
use vize_extension_host::{SurfacePage, TileError};
use vize_s0::{Allocator, String};

fn rendered(tree: &vize_s1::SurfaceTree<'_>) -> String {
    let mut out = String::default();
    vize_s1::render(tree, &mut |piece| out.push_str(piece));
    out
}

fn assert_laws(source: &str, context: &str) {
    let allocator = Allocator::new();
    let (tree, _errors) = vize_s1::parse(&allocator, source);
    let page = SurfacePage::of(&tree);
    let text = full_text(&page);
    let parsed = SurfacePage::parse(&text).unwrap_or_else(|error| panic!("{context}: {error}"));
    assert_eq!(parsed, page, "parse(print(page)) != page: {context}");
    assert_eq!(
        full_text(&parsed),
        text,
        "print(parse(text)) != text: {context}"
    );
    assert_eq!(page.bytes() as usize, source.len(), "bytes=: {context}");

    let rebuilt_arena = Allocator::new();
    let rebuilt = parsed
        .materialize(&rebuilt_arena, source)
        .unwrap_or_else(|error| panic!("{context}: {error}"));
    assert_eq!(
        rendered(&rebuilt).as_str(),
        source,
        "render(materialize) != source: {context}"
    );
    assert_eq!(
        SurfacePage::of(&rebuilt),
        page,
        "page(materialize) != page: {context}"
    );
}

#[test]
fn battery_and_truncations_obey_the_page_laws() {
    assert_eq!(WELL_FORMED.len() + MALFORMED.len(), 42);
    let mut checked = 0usize;
    for fixture in WELL_FORMED.iter().chain(MALFORMED) {
        let source = fixture.source;
        assert_laws(source, fixture.name);
        checked += 1;
        for cut in (0..source.len()).filter(|&cut| source.is_char_boundary(cut)) {
            assert_laws(&source[..cut], fixture.name);
            assert_laws(&source[cut..], fixture.name);
            checked += 2;
        }
    }
    assert_eq!(checked, 2_148, "the truncation hammer's scope is pinned");
}

const REFERENCE_SOURCE: &str = "<a b='1' c>{{ x }}</a><br/><!-- c --><p>t";
const REFERENCE_PAGE: &str = "[s1]
bytes=41

[s1.tree]
element
  lt-name 0:0:2
  attr
    name 2:3:4
    eq 4:4:5
    open-quote 5:5:6
    value 6:6:7
    close-quote 7:7:8
  attr
    name 8:9:10
  gt 10:10:11
  interpolation
    open 11:11:13
    content 13:13:16
    close 16:16:18
  close-tag
    lt-slash-name 18:18:21
    gt 21:21:22
element
  lt-name 22:22:25
  slash 25:25:26
  gt 26:26:27
  close not-expected
comment 27:27:37
element
  lt-name 37:37:39
  gt 39:39:40
  text 40:40:41
  close missing

";

#[test]
fn the_reference_page_is_pinned() {
    let allocator = Allocator::new();
    let (tree, _errors) = vize_s1::parse(&allocator, REFERENCE_SOURCE);
    assert_eq!(full_text(&SurfacePage::of(&tree)).as_str(), REFERENCE_PAGE);
}

#[test]
fn missing_tokens_and_the_empty_page_print_exactly() {
    let allocator = Allocator::new();
    let (tree, _errors) = vize_s1::parse(&allocator, "<b title=\"x");
    assert_eq!(
        full_text(&SurfacePage::of(&tree)).as_str(),
        "[s1]\nbytes=11\n\n[s1.tree]\nelement\n  lt-name 0:0:2\n  attr\n    name 2:3:8\n    \
         eq 8:8:9\n    open-quote 9:9:10\n    value 10:10:11\n    close-quote 11:11:11 missing\n  \
         gt 11:11:11 missing\n  close missing\n\n"
    );
    let (empty, _errors) = vize_s1::parse(&allocator, "");
    assert_eq!(
        full_text(&SurfacePage::of(&empty)).as_str(),
        "[s1]\nbytes=0\n\n"
    );
}

fn refusal(text: &str) -> FolioError {
    SurfacePage::parse(text).expect_err("the page must be refused")
}

#[test]
fn malformed_pages_are_refused_exactly() {
    let cases: &[(&str, usize, &str)] = &[
        ("bytes=1\n", 1, "content before the [s1] header"),
        ("[disegno]\n", 1, "first section must be [s1]"),
        ("[s1]\n", 0, "missing field `bytes`"),
        ("[s1]\nbytes=x\n", 2, "invalid integer `x`"),
        ("[s1]\nbytes=1\nbytes=1\n", 3, "duplicate field `bytes`"),
        ("[s1]\nops=1\n", 2, "unknown field line `ops=1`"),
        ("[s1]\nbytes=0\n[s1.ops]\n", 3, "unknown section [s1.ops]"),
        (
            "[s1]\nbytes=0\n[s1.tree]\n[s1.tree]\n",
            4,
            "duplicate section [s1.tree]",
        ),
        (
            "[s1]\nbytes=1\n[s1.tree]\n text 0:0:1\n",
            4,
            "odd indentation",
        ),
        (
            "[s1]\nbytes=1\n[s1.tree]\n  text 0:0:1\n",
            4,
            "over-indented line",
        ),
        ("[s1]\nbytes=1\n[s1.tree]\ngt 0:0:1\n", 4, "expected a node"),
        (
            "[s1]\nbytes=1\n[s1.tree]\ntext 0:1\n",
            4,
            "invalid offset `` in `0:1`",
        ),
        (
            "[s1]\nbytes=1\n[s1.tree]\ntext 0:0:1:2\n",
            4,
            "a token is exactly `start:text:end`",
        ),
        (
            "[s1]\nbytes=1\n[s1.tree]\ntext 1:0:1\n",
            4,
            "token offsets out of order in `1:0:1`",
        ),
        (
            "[s1]\nbytes=1\n[s1.tree]\ntext 0:0:1 absent\n",
            4,
            "unknown token flag `absent`",
        ),
        (
            "[s1]\nbytes=1\n[s1.tree]\ntext 0:0:1 missing\n",
            4,
            "a missing token is zero-width",
        ),
        (
            "[s1]\nbytes=1\n[s1.tree]\nelement x\n",
            4,
            "`element` takes no operands",
        ),
        (
            "[s1]\nbytes=2\n[s1.tree]\nelement\n  gt 0:0:1\n",
            5,
            "expected `lt-name`",
        ),
        (
            "[s1]\nbytes=2\n[s1.tree]\nelement\n  lt-name 0:0:1\n  gt 1:1:2\n  lt-name 2:2:2\n",
            7,
            "expected an element child or `close`",
        ),
        (
            "[s1]\nbytes=2\n[s1.tree]\nelement\n  lt-name 0:0:1\n",
            0,
            "unexpected end of input: expected `gt`",
        ),
    ];
    for &(text, line, message) in cases {
        assert_eq!(
            refusal(text),
            FolioError::new(line, String::from(message)),
            "{text:?}"
        );
    }
}

fn message(result: Result<(), TileError>) -> Result<(), String> {
    result.map_err(|error| vize_s0::cstr!("{error}"))
}

fn tiles(text: &str, source: &str) -> Result<(), TileError> {
    SurfacePage::parse(text)
        .expect("the page parses")
        .check_tiles(source)
}

#[test]
fn pages_that_do_not_tile_their_source_are_refused_exactly() {
    let head = "[s1]\nbytes=0\n[s1.tree]\n";
    let gap = [head, "text 0:0:2\ntext 3:3:4\n"].concat();
    assert_eq!(
        message(tiles(&gap, "abcd")),
        Err(String::from("token 1 (`text`) starts at 3, expected 2"))
    );
    let short = [head, "text 0:0:3\n"].concat();
    assert_eq!(
        tiles(&short, "abcd"),
        Err(TileError::Short { end: 3, len: 4 })
    );
    let split = [head, "text 0:0:1\n"].concat();
    assert_eq!(
        message(tiles(&split, "é")),
        Err(String::from(
            "token 0 (`text`) offset 1 is not a character boundary of the block"
        ))
    );
    let past = [head, "text 0:0:9\n"].concat();
    assert_eq!(
        tiles(&past, "abcd"),
        Err(TileError::Boundary {
            index: 0,
            role: "text",
            offset: 9
        })
    );
    let allocator = Allocator::new();
    assert_eq!(
        SurfacePage::parse(&gap)
            .expect("the page parses")
            .materialize(&allocator, "abcd")
            .map(|_| ()),
        Err(TileError::Gap {
            index: 1,
            role: "text",
            start: 3,
            expected: 2
        })
    );
    let token = PageToken {
        start: 0,
        text: 2,
        end: 1,
        missing: false,
    };
    let backwards = SurfacePage {
        children: vec![PageNode::Text(token)],
    };
    assert_eq!(
        message(backwards.check_tiles("ab")),
        Err(String::from(
            "token 0 (`text`) is malformed: offsets out of order or a missing token with width"
        ))
    );
    let wide_hole = SurfacePage {
        children: vec![PageNode::Unexpected(PageToken {
            start: 0,
            text: 0,
            end: 2,
            missing: true,
        })],
    };
    assert_eq!(
        wide_hole.check_tiles("ab"),
        Err(TileError::Malformed {
            index: 0,
            role: "unexpected"
        })
    );
}
