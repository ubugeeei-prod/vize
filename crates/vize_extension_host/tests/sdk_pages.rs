//! The SDK's page writers speak the host's grammar exactly (P6-2).
//!
//! The SDK cannot depend on the host, so the L1 page has two printers — the
//! host's (`SurfacePage`) and the guest's (`vize_extension_sdk::pages::l1`).
//! Over the TS-19 battery and its truncations they print byte-equal pages;
//! the L2 writer's pages are canonical disegno pages the host accepts; and
//! the SDK's capability offer negotiates exactly as the in-tree one.

#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use davinci_test_support::surface_fixture::{MALFORMED, WELL_FORMED};
use vize_davinci::folio::Folio;
use vize_extension_host::accept::full_text;
use vize_extension_host::surface_page::{PageClose, PageNode, PageToken};
use vize_extension_host::{SurfacePage, negotiate};
use vize_extension_sdk::pages::{l1, l2};
use vize_extension_sdk::types::Span;
use vize_l0::{Allocator, String};
use vize_l2::folio::L2Folio;

fn token(token: &PageToken) -> l1::Token {
    l1::Token {
        start: token.start,
        text: token.text,
        end: token.end,
        missing: token.missing,
    }
}

fn mirror(node: &PageNode) -> l1::Node {
    match node {
        PageNode::Element(element) => l1::Node::Element(l1::Element {
            lt_name: token(&element.lt_name),
            attrs: element
                .attrs
                .iter()
                .map(|attr| l1::Attribute {
                    name: token(&attr.name),
                    eq: attr.eq.as_ref().map(token),
                    value: attr.value.as_ref().map(|value| l1::AttrValue {
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
                PageClose::Tag { lt_slash_name, gt } => l1::Close::Tag {
                    lt_slash_name: token(lt_slash_name),
                    gt: token(gt),
                },
                PageClose::Implicit => l1::Close::Implicit,
                PageClose::Missing => l1::Close::Missing,
                PageClose::NotExpected => l1::Close::NotExpected,
            },
        }),
        PageNode::Interpolation(node) => l1::Node::Interpolation {
            open: token(&node.open),
            content: token(&node.content),
            close: token(&node.close),
        },
        PageNode::Text(t) => l1::Node::Text(token(t)),
        PageNode::Comment(t) => l1::Node::Comment(token(t)),
        PageNode::Cdata(t) => l1::Node::Cdata(token(t)),
        PageNode::ProcessingInstruction(t) => l1::Node::ProcessingInstruction(token(t)),
        PageNode::Unexpected(t) => l1::Node::Unexpected(token(t)),
    }
}

fn assert_same_l1(source: &str, context: &str) {
    let allocator = Allocator::new();
    let (tree, _errors) = vize_l1::parse(&allocator, source);
    let host = SurfacePage::of(&tree);
    let guest = l1::SurfacePage {
        children: host.children.iter().map(mirror).collect(),
    };
    assert_eq!(
        guest.text().as_str(),
        full_text(&host).as_str(),
        "{context}"
    );
}

#[test]
fn the_sdk_l1_writer_prints_the_host_grammar() {
    let mut checked = 0usize;
    for fixture in WELL_FORMED.iter().chain(MALFORMED) {
        let source = fixture.source;
        assert_same_l1(source, fixture.name);
        checked += 1;
        for cut in (0..source.len()).filter(|&cut| source.is_char_boundary(cut)) {
            assert_same_l1(&source[..cut], fixture.name);
            assert_same_l1(&source[cut..], fixture.name);
            checked += 2;
        }
    }
    assert_eq!(checked, 2_148);
}

#[test]
fn the_sdk_l2_writer_prints_canonical_disegno_pages() {
    let at = |start, end| Span { start, end };
    let page = l2::SemanticPage {
        ops: vec![
            l2::Op::Element {
                tag: "section".into(),
                attrs: vec![
                    l2::Attr {
                        name: "hidden".into(),
                        value: None,
                        span: at(9, 15),
                    },
                    l2::Attr {
                        name: "title".into(),
                        value: Some("a \"q\"\\\n\t\r".into()),
                        span: at(16, 30),
                    },
                ],
                children: vec![l2::Op::Text {
                    value: "héllo 👋".into(),
                    span: at(31, 42),
                }],
                span: at(0, 52),
            },
            l2::Op::Text {
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
    let parsed = L2Folio::parse(&text).expect("the page parses");
    assert_eq!(full_text(&parsed).as_str(), text);
    let empty = l2::SemanticPage::default().text();
    assert_eq!(empty, "[disegno]\nops=0\n\n");
    assert_eq!(
        full_text(&L2Folio::parse(&empty).expect("parses")).as_str(),
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

#[test]
fn legacy_sdk_names_write_identical_wire_pages() {
    use vize_extension_sdk::{L1_PAGE_SCHEMA, L2_PAGE_SCHEMA, L3_PAGE_SCHEMA, pages};

    let surface: pages::s1::SurfacePage = pages::l1::SurfacePage::default();
    let semantic: pages::s2::SemanticPage = pages::l2::SemanticPage::default();
    assert_eq!(surface.text(), "[s1]\nbytes=0\n\n");
    assert_eq!(semantic.text(), "[disegno]\nops=0\n\n");
    assert_eq!(
        pages::s1_page(surface.text()),
        pages::l1_page(surface.text())
    );
    assert_eq!(
        pages::s2_page(semantic.text()),
        pages::l2_page(semantic.text())
    );
    assert_eq!(vize_extension_sdk::S1_PAGE_SCHEMA, L1_PAGE_SCHEMA);
    assert_eq!(vize_extension_sdk::S2_PAGE_SCHEMA, L2_PAGE_SCHEMA);
    assert_eq!(vize_extension_sdk::S3_PAGE_SCHEMA, L3_PAGE_SCHEMA);
    assert_eq!(
        vize_extension_host::contract::S1_PAGE_SCHEMA,
        L1_PAGE_SCHEMA
    );
    assert_eq!(
        vize_extension_host::contract::S2_PAGE_SCHEMA,
        L2_PAGE_SCHEMA
    );
    assert_eq!(vize_extension_host::output::S3_PAGE_SCHEMA, L3_PAGE_SCHEMA);
}
