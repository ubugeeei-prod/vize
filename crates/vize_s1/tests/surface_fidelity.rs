//! TS-19 — S1 byte fidelity, the plain (default-lane) suite.
//!
//! `render(parse(src)) == src` as **bytes** over the committed battery —
//! well-formed and malformed, the `Unexpected`/`Missing` paths included —
//! plus every prefix and suffix truncation of every fixture (the cheap
//! deterministic EOF-recovery hammer), with the typed-hole census pinned
//! exactly per fixture. Exact-equality oracles only (assurance §4).

use davinci_test_support::surface_fixture as common;
use vize_relief::ErrorCode;
use vize_s0::{Allocator, String};
use vize_s1::{
    ElementClose, HoleCounts, SurfaceChild, SurfaceTree, check_fidelity, hole_counts, parse, render,
};

fn rendered(tree: &SurfaceTree<'_>) -> String {
    let mut out = String::default();
    render(tree, &mut |piece| out.push_str(piece));
    out
}

fn assert_fidelity(source: &str, context: &str) {
    let allocator = Allocator::new();
    let (tree, _errors) = parse(&allocator, source);
    let out = rendered(&tree);
    assert_eq!(out.as_str(), source, "render(parse(src)) != src: {context}");
    assert_eq!(check_fidelity(&tree), Ok(()), "fidelity check: {context}");
}

#[test]
fn ts19_battery_round_trips_with_pinned_holes() {
    // Exact-pinned scope: a battery change must move these deliberately.
    assert_eq!(common::WELL_FORMED.len(), 16);
    assert_eq!(common::MALFORMED.len(), 26);
    for fixture in common::WELL_FORMED.iter().chain(common::MALFORMED) {
        let allocator = Allocator::new();
        let (tree, _errors) = parse(&allocator, fixture.source);
        let out = rendered(&tree);
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
}

#[test]
fn ts19_truncations_round_trip() {
    for fixture in common::WELL_FORMED.iter().chain(common::MALFORMED) {
        let src = fixture.source;
        for (idx, _) in src.char_indices() {
            assert_fidelity(&src[..idx], fixture.name);
            assert_fidelity(&src[idx..], fixture.name);
        }
        assert_fidelity(src, fixture.name);
    }
}

#[test]
fn eof_in_tag_holes_are_typed() {
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, "<div");
    assert_eq!(tree.children.len(), 1);
    let SurfaceChild::Element(element) = &tree.children[0] else {
        panic!("`<div` parses to one element");
    };
    assert_eq!(element.tag(), "div");
    assert!(element.open.gt.is_missing());
    assert!(matches!(element.close, ElementClose::Missing));
    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0].code, ErrorCode::EofInTag));
}

#[test]
fn stray_close_tag_is_a_verbatim_unexpected_node() {
    let allocator = Allocator::new();
    let (tree, _errors) = parse(&allocator, "</div>");
    assert_eq!(tree.children.len(), 1);
    let SurfaceChild::Unexpected(token) = &tree.children[0] else {
        panic!("a stray end tag is an Unexpected hole");
    };
    assert_eq!(token.text, "</div>");
    assert!(!token.is_missing());
}

#[test]
fn unterminated_quote_is_a_missing_token() {
    let allocator = Allocator::new();
    let (tree, _errors) = parse(&allocator, "<div a=\"x");
    let SurfaceChild::Element(element) = &tree.children[0] else {
        panic!("one element");
    };
    assert_eq!(element.open.attrs.len(), 1);
    let attr = &element.open.attrs[0];
    assert_eq!(attr.name.text, "a");
    assert_eq!(attr.eq.as_ref().map(|token| token.text), Some("="));
    let value = attr.value.as_ref().expect("the value exists");
    assert_eq!(
        value.open_quote.as_ref().map(|token| token.text),
        Some("\"")
    );
    assert_eq!(value.content.text, "x");
    let close_quote = value.close_quote.as_ref().expect("modelled, as a hole");
    assert!(close_quote.is_missing());
}

#[test]
fn self_closing_and_void_elements_expect_no_close() {
    let allocator = Allocator::new();
    let (tree, _errors) = parse(&allocator, "<br /><img src=\"x\">t");
    assert_eq!(tree.children.len(), 3);
    let SurfaceChild::Element(br) = &tree.children[0] else {
        panic!("children[0] is <br />");
    };
    assert_eq!(br.open.slash.as_ref().map(|token| token.text), Some("/"));
    assert!(matches!(br.close, ElementClose::NotExpected));
    let SurfaceChild::Element(img) = &tree.children[1] else {
        panic!("children[1] is <img>");
    };
    assert!(img.open.slash.is_none());
    assert!(matches!(img.close, ElementClose::NotExpected));
    assert!(matches!(&tree.children[2], SurfaceChild::Text(_)));
}

#[test]
fn nested_interactive_content_keeps_fidelity_without_missing_close_holes() {
    for source in [
        r#"<div><a href="/"><div><a href="/foo">inner</a></div></a></div>"#,
        "<div><button><div><button>bbb</button></div></button></div>",
    ] {
        let allocator = Allocator::new();
        let (tree, _errors) = parse(&allocator, source);
        assert_eq!(rendered(&tree), source);
        assert_eq!(check_fidelity(&tree), Ok(()));
        assert_eq!(
            hole_counts(&tree),
            HoleCounts {
                missing_tokens: 0,
                missing_close_tags: 0,
                unexpected_nodes: 2,
            },
            "{source}: the redundant descendant and interactive end tags are kept as surface holes"
        );
    }
}

#[test]
fn direct_nested_interactive_end_tag_closes_live_inner_before_redundant_outer() {
    assert_direct_nested_interactive_close_order("<a><a>x</a></a>", "a", "</a", "</a>");
    assert_direct_nested_interactive_close_order(
        "<button><button>x</button></button>",
        "button",
        "</button",
        "</button>",
    );
}

fn assert_direct_nested_interactive_close_order(
    source: &str,
    tag: &str,
    close_prefix: &str,
    redundant_close: &str,
) {
    let allocator = Allocator::new();
    let (tree, _errors) = parse(&allocator, source);
    assert_eq!(rendered(&tree), source);
    assert_eq!(check_fidelity(&tree), Ok(()));
    assert_eq!(
        hole_counts(&tree),
        HoleCounts {
            missing_tokens: 0,
            missing_close_tags: 0,
            unexpected_nodes: 1,
        }
    );
    assert_eq!(tree.children.len(), 3);

    let SurfaceChild::Element(outer) = &tree.children[0] else {
        panic!("{source}: first child is the implicitly closed outer element");
    };
    assert_eq!(outer.tag(), tag);
    assert!(outer.children.is_empty());
    assert!(matches!(outer.close, ElementClose::Implicit));

    let SurfaceChild::Element(inner) = &tree.children[1] else {
        panic!("{source}: second child is the live inner element");
    };
    assert_eq!(inner.tag(), tag);
    assert!(matches!(&inner.children[0], SurfaceChild::Text(text) if text.text == "x"));
    let ElementClose::Present(close) = &inner.close else {
        panic!("{source}: inner element must own the first authored end tag");
    };
    assert_eq!(close.lt_slash_name.text, close_prefix);
    assert_eq!(close.gt.text, ">");

    let SurfaceChild::Unexpected(token) = &tree.children[2] else {
        panic!("{source}: redundant outer end tag is the only Unexpected hole");
    };
    assert_eq!(token.text, redundant_close);
}

#[test]
fn nested_interactive_recovery_stays_in_html_namespace() {
    for source in [
        r#"<A><a href="/foo">inner</a></A>"#,
        "<svg><a><a>x</a></a></svg>",
    ] {
        let allocator = Allocator::new();
        let (tree, _errors) = parse(&allocator, source);
        assert_eq!(rendered(&tree), source);
        assert_eq!(check_fidelity(&tree), Ok(()));
        assert_eq!(
            hole_counts(&tree),
            HoleCounts::default(),
            "{source}: component-like or foreign anchors must keep authored nesting"
        );
    }
}

#[test]
fn interpolation_tokens_are_delimited() {
    let allocator = Allocator::new();
    let (tree, _errors) = parse(&allocator, "a{{ msg }}b");
    assert_eq!(tree.children.len(), 3);
    let SurfaceChild::Interpolation(node) = &tree.children[1] else {
        panic!("children[1] is the interpolation");
    };
    assert_eq!(node.open.text, "{{");
    assert_eq!(node.content.text, " msg ");
    assert_eq!(node.close.text, "}}");
}

#[test]
fn attribute_tokens_split_exactly() {
    let allocator = Allocator::new();
    let (tree, _errors) = parse(&allocator, "<a b=\"1\" c='2' d=3 e>t</a>");
    let SurfaceChild::Element(element) = &tree.children[0] else {
        panic!("one element");
    };
    assert_eq!(element.open.attrs.len(), 4);
    let b = &element.open.attrs[0];
    let b_value = b.value.as_ref().expect("b has a value");
    assert_eq!(b.name.leading, " ");
    assert_eq!(
        b_value.open_quote.as_ref().map(|token| token.text),
        Some("\"")
    );
    assert_eq!(b_value.content.text, "1");
    assert_eq!(
        b_value.close_quote.as_ref().map(|token| token.text),
        Some("\"")
    );
    let c_value = element.open.attrs[1].value.as_ref().expect("c has a value");
    assert_eq!(
        c_value.open_quote.as_ref().map(|token| token.text),
        Some("'")
    );
    let d = &element.open.attrs[2];
    let d_value = d.value.as_ref().expect("d has a value");
    assert_eq!(d.eq.as_ref().map(|token| token.text), Some("="));
    assert!(d_value.open_quote.is_none());
    assert_eq!(d_value.content.text, "3");
    assert!(d_value.close_quote.is_none());
    let e = &element.open.attrs[3];
    assert!(e.eq.is_none());
    assert!(e.value.is_none());
}

#[test]
fn directive_names_stay_raw() {
    let allocator = Allocator::new();
    let (tree, _errors) = parse(&allocator, "<a v-on:click.stop=\"f\" :[key]=\"v\" #s>x</a>");
    let SurfaceChild::Element(element) = &tree.children[0] else {
        panic!("one element");
    };
    assert_eq!(element.open.attrs.len(), 3);
    assert_eq!(element.open.attrs[0].name.text, "v-on:click.stop");
    assert_eq!(element.open.attrs[1].name.text, ":[key]");
    assert_eq!(element.open.attrs[2].name.text, "#s");
}

#[test]
fn in_tag_junk_rides_in_leading_under_a_diagnostic() {
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, "<div / a>x</div>");
    let SurfaceChild::Element(element) = &tree.children[0] else {
        panic!("one element");
    };
    // Hole policy clause 3: the stray `/` is leading, not structure —
    // the element is *not* self-closing and the diagnostic names it.
    assert!(element.open.slash.is_none());
    assert_eq!(element.open.attrs[0].name.leading, " / ");
    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0].code, ErrorCode::UnexpectedSolidusInTag));
}
