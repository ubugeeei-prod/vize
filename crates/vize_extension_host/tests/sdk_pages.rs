//! The SDK's page writers speak the host's grammar exactly (P6-2).
//!
//! The SDK cannot depend on the host, so the S1 page has two printers — the
//! host's (`SurfacePage`) and the guest's (`vize_extension_sdk::pages::s1`).
//! Over the TS-19 battery and its truncations they print byte-equal pages;
//! the S2 writer's pages are canonical disegno pages the host accepts; and
//! the SDK's capability offer negotiates exactly as the in-tree one.

#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use davinci_test_support::surface_fixture::{MALFORMED, WELL_FORMED};
use vize_davinci::folio::Folio;
use vize_extension_host::accept::full_text;
use vize_extension_host::surface_page::{PageClose, PageNode, PageToken};
use vize_extension_host::{SurfacePage, negotiate};
use vize_extension_sdk::pages::{s1, s2};
use vize_extension_sdk::types::Span;
use vize_s0::{Allocator, String};
use vize_s2::folio::S2Folio;

fn token(token: &PageToken) -> s1::Token {
    s1::Token {
        start: token.start,
        text: token.text,
        end: token.end,
        missing: token.missing,
    }
}

fn mirror(node: &PageNode) -> s1::Node {
    match node {
        PageNode::Element(element) => s1::Node::Element(s1::Element {
            lt_name: token(&element.lt_name),
            attrs: element
                .attrs
                .iter()
                .map(|attr| s1::Attribute {
                    name: token(&attr.name),
                    eq: attr.eq.as_ref().map(token),
                    value: attr.value.as_ref().map(|value| s1::AttrValue {
                        open_quote: value.open_quote.as_ref().map(token),
                        content: token(&value.content),
                        close_quote: value.close_quote.as_ref().map(token),
                    }),
                })
                .collect(),
            slash: element.slash.as_ref().map(token),
            gt: token(&element.gt),
            children: element.children.iter().map(mirror).collect(),
            close: match &element.close {
                PageClose::Tag { lt_slash_name, gt } => s1::Close::Tag {
                    lt_slash_name: token(lt_slash_name),
                    gt: token(gt),
                },
                PageClose::Implicit => s1::Close::Implicit,
                PageClose::Missing => s1::Close::Missing,
                PageClose::NotExpected => s1::Close::NotExpected,
            },
        }),
        PageNode::Interpolation(node) => s1::Node::Interpolation {
            open: token(&node.open),
            content: token(&node.content),
            close: token(&node.close),
        },
        PageNode::Text(t) => s1::Node::Text(token(t)),
        PageNode::Comment(t) => s1::Node::Comment(token(t)),
        PageNode::Cdata(t) => s1::Node::Cdata(token(t)),
        PageNode::ProcessingInstruction(t) => s1::Node::ProcessingInstruction(token(t)),
        PageNode::Unexpected(t) => s1::Node::Unexpected(token(t)),
    }
}

fn assert_same_s1(source: &str, context: &str) {
    let allocator = Allocator::new();
    let (tree, _errors) = vize_s1::parse(&allocator, source);
    let host = SurfacePage::of(&tree);
    let guest = s1::SurfacePage {
        children: host.children.iter().map(mirror).collect(),
    };
    assert_eq!(
        guest.text().as_str(),
        full_text(&host).as_str(),
        "{context}"
    );
}

#[test]
fn the_sdk_s1_writer_prints_the_host_grammar() {
    let mut checked = 0usize;
    for fixture in WELL_FORMED.iter().chain(MALFORMED) {
        let source = fixture.source;
        assert_same_s1(source, fixture.name);
        checked += 1;
        for cut in (0..source.len()).filter(|&cut| source.is_char_boundary(cut)) {
            assert_same_s1(&source[..cut], fixture.name);
            assert_same_s1(&source[cut..], fixture.name);
            checked += 2;
        }
    }
    assert_eq!(checked, 2_148);
}

#[test]
fn the_sdk_s2_writer_prints_canonical_disegno_pages() {
    let at = |start, end| Span { start, end };
    let page = s2::SemanticPage {
        ops: vec![
            s2::Op::Element {
                tag: "section".into(),
                attrs: vec![
                    s2::Attr {
                        name: "hidden".into(),
                        value: None,
                        span: at(9, 15),
                    },
                    s2::Attr {
                        name: "title".into(),
                        value: Some("a \"q\"\\\n\t\r".into()),
                        span: at(16, 30),
                    },
                ],
                children: vec![s2::Op::Text {
                    value: "héllo 👋".into(),
                    span: at(31, 42),
                }],
                span: at(0, 52),
            },
            s2::Op::Text {
                value: "".into(),
                span: at(52, 52),
            },
        ],
    };
    let text = page.text();
    assert_eq!(
        text,
        "[disegno]\nops=3\n\n[disegno.ops]\nui.element section @0:52\n  attr hidden @9:15\n  \
         attr title=\"a \\\"q\\\"\\\\\\n\\t\\r\" @16:30\n  ui.text \"héllo 👋\" @31:42\nui.text \"\" @52:52\n\n"
    );
    let parsed = S2Folio::parse(&text).expect("the page parses");
    assert_eq!(full_text(&parsed).as_str(), text);
    let empty = s2::SemanticPage::default().text();
    assert_eq!(empty, "[disegno]\nops=0\n\n");
    assert_eq!(
        full_text(&S2Folio::parse(&empty).expect("parses")).as_str(),
        empty
    );
}

#[test]
fn the_sdk_capability_negotiates_like_the_in_tree_offer() {
    let offer = vize_extension_sdk::capability(&["html"]);
    let mirrored = vize_extension_host::Capability {
        protocol_version: offer.protocol_version,
        features: offer
            .features
            .iter()
            .map(|f| String::from(f.as_str()))
            .collect(),
    };
    assert_eq!(mirrored, vize_extension_host::vue::capability());
    let negotiated = negotiate(&mirrored).expect("negotiates");
    assert_eq!(negotiated.langs, [String::from("html")]);
}
