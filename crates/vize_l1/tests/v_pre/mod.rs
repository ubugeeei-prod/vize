//! Lexical `v-pre` scope over the lossless surface tree.

use super::rendered;
use vize_l0::Allocator;
use vize_l1::{SurfaceChild, check_fidelity, parse};

#[test]
fn v_pre_scope_follows_implicit_interactive_closure() {
    for source in [
        "<a v-pre><a>{{ inner }}</a>{{ after }}",
        "<button v-pre><button>{{ inner }}</button>{{ after }}",
        "<a v-pre><span><a>{{ inner }}</a>{{ after }}",
    ] {
        let allocator = Allocator::new();
        let (tree, _errors) = parse(&allocator, source);
        assert_eq!(rendered(&tree), source);
        assert_eq!(check_fidelity(&tree), Ok(()));
        assert!(
            matches!(tree.children.last(), Some(SurfaceChild::Interpolation(_))),
            "{source}"
        );
    }

    // SVG anchors do not follow the HTML interactive close rule.
    let source = "<svg><a v-pre><a>{{ inner }}</a>{{ after }}</a></svg>";
    let allocator = Allocator::new();
    let (tree, _errors) = parse(&allocator, source);
    assert_eq!(rendered(&tree), source);
    let SurfaceChild::Element(svg) = &tree.children[0] else {
        panic!("root is svg");
    };
    let SurfaceChild::Element(outer) = &svg.children[0] else {
        panic!("svg child is outer a");
    };
    assert!(matches!(outer.children.last(), Some(SurfaceChild::Text(_))));
}

#[test]
fn v_pre_lexes_mustaches_as_text_until_its_element_closes() {
    let source = "<div v-pre>{{ a }}<span>{{ b }}</span></div>{{ c }}";
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, source);
    assert!(errors.is_empty());
    assert_eq!(rendered(&tree), source);
    let SurfaceChild::Element(div) = &tree.children[0] else {
        panic!("first child is the v-pre element");
    };
    assert!(matches!(&div.children[0], SurfaceChild::Text(text) if text.text == "{{ a }}"));
    let SurfaceChild::Element(span) = &div.children[1] else {
        panic!("nested span stays structural");
    };
    assert!(matches!(&span.children[0], SurfaceChild::Text(text) if text.text == "{{ b }}"));
    assert!(
        matches!(&tree.children[1], SurfaceChild::Interpolation(node) if node.content.text == " c ")
    );
}

#[test]
fn v_pre_lexical_scope_excludes_void_and_self_closing_tags() {
    for source in [
        "<br v-pre>{{ after }}",
        "<div v-pre/>{{ after }}",
        "<div v-pre><br>{{ inside }}</div>{{ after }}",
    ] {
        let allocator = Allocator::new();
        let (tree, errors) = parse(&allocator, source);
        assert!(errors.is_empty(), "{source}");
        assert_eq!(rendered(&tree), source);
        assert!(
            matches!(tree.children.last(), Some(SurfaceChild::Interpolation(node)) if node.content.text == " after "),
            "{source}"
        );
        if source.starts_with("<div v-pre><br") {
            let SurfaceChild::Element(div) = &tree.children[0] else {
                panic!("first child is the div");
            };
            assert!(
                matches!(&div.children[1], SurfaceChild::Text(text) if text.text == "{{ inside }}")
            );
        }
    }
}

#[test]
fn only_the_exact_v_pre_attribute_switches_lexing_mode() {
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, "<div v-pre.foo>{{ expression }}</div>");
    assert!(errors.is_empty());
    let SurfaceChild::Element(div) = &tree.children[0] else {
        panic!("first child is the div");
    };
    assert!(
        matches!(&div.children[0], SurfaceChild::Interpolation(node) if node.content.text == " expression ")
    );
}

#[test]
fn v_pre_scope_recovers_at_a_matching_ancestor_close() {
    let source = "<section v-pre><p>{{ inside }}</section>{{ after }}";
    let allocator = Allocator::new();
    let (tree, _errors) = parse(&allocator, source);
    assert_eq!(rendered(&tree), source);
    assert!(
        matches!(tree.children.last(), Some(SurfaceChild::Interpolation(node)) if node.content.text == " after ")
    );
    let SurfaceChild::Element(section) = &tree.children[0] else {
        panic!("first child is section");
    };
    let SurfaceChild::Element(paragraph) = &section.children[0] else {
        panic!("paragraph remains in section");
    };
    assert!(
        matches!(&paragraph.children[0], SurfaceChild::Text(text) if text.text == "{{ inside }}")
    );
}
