//! Ordinary-library native construction: structure, recovery and source bytes.

#![cfg(test)]

use davinci_test_support::surface_fixture as common;
use vize_l0::{Allocator, ErrorCode, cstr};
use vize_l1::markup::{
    parse_component, parse_component_with_authored, parse_component_with_options,
};
use vize_l1::{
    HoleCounts, SurfaceChild, SurfaceParseOptions, check_fidelity, hole_counts, parse,
    parse_with_authored, parse_with_options,
};

fn assert_parity(source: &str, options: SurfaceParseOptions) {
    let allocator = Allocator::new();
    let parsed = parse_component_with_options(&allocator, source, options).unwrap();
    assert!(parsed.unsupported.is_empty(), "{source}");
    let (native, native_errors) = (parsed.tree, parsed.errors);
    let (compat, compat_errors) = parse_with_options(&allocator, source, options);
    // Debug includes every node, authored slice, token leading and hole status.
    assert_eq!(cstr!("{native:?}"), cstr!("{compat:?}"), "{source}");
    assert_eq!(
        cstr!("{native_errors:?}"),
        cstr!("{compat_errors:?}"),
        "ordered diagnostic codes and offsets: {source}"
    );
    assert_eq!(hole_counts(&native), hole_counts(&compat), "{source}");
    assert_eq!(check_fidelity(&native), Ok(()), "{source}");
}

#[test]
fn native_public_constructor_matches_all_fixture_trees_and_pinned_holes() {
    assert_eq!(common::WELL_FORMED.len(), 16);
    assert_eq!(common::MALFORMED.len(), 26);
    for fixture in common::WELL_FORMED.iter().chain(common::MALFORMED) {
        assert_parity(fixture.source, SurfaceParseOptions::default());
        let allocator = Allocator::new();
        let tree = parse_component(&allocator, fixture.source).unwrap().tree;
        assert_eq!(
            hole_counts(&tree),
            HoleCounts {
                missing_tokens: fixture.missing_tokens,
                missing_close_tags: fixture.missing_close_tags,
                unexpected_nodes: fixture.unexpected_nodes,
            },
            "{}",
            fixture.name
        );
    }
}

#[test]
fn native_public_constructor_keeps_exact_recovery_at_all_utf8_cuts() {
    for fixture in common::WELL_FORMED.iter().chain(common::MALFORMED) {
        for (index, _) in fixture.source.char_indices() {
            for source in [
                fixture.source.get(..index).expect("UTF-8 boundary"),
                fixture.source.get(index..).expect("UTF-8 boundary"),
            ] {
                assert_parity(source, SurfaceParseOptions::default());
            }
        }
    }
}

#[test]
fn native_entities_quotes_and_in_tag_comment_switch_keep_authored_bytes() {
    for source in [
        "<p title='&fjlig;&acE;&nGt;'>&fjlig;&acE;&nGt;</p>",
        "<p title='&amp;#40; &timesX'>&timesX &#x80; &#0;</p>",
        "<div title=\"日",
        "<div a= >{{ unfinished",
        "\r\n<!-- 日😀 --><?proc?><x />",
    ] {
        assert_parity(source, SurfaceParseOptions::default());
    }
    assert_parity(
        "<Comp // @vue-expect-error API\n :title='日'/>",
        SurfaceParseOptions {
            experimental_in_tag_comments: true,
        },
    );
}

#[test]
fn native_authored_projection_shares_normal_recovery_and_diagnostics() {
    for source in [
        "<a><span><a>日本語😀</a></span></a>",
        "<button><button x='unterminated",
        "<svg><a><a>foreign</a></a></svg>",
        "<math><button><button>foreign</button></button></math>",
        "<a><a broken= >{{ }}<span",
    ] {
        let allocator = Allocator::new();
        let native = parse_component_with_authored(&allocator, source).unwrap();
        let compat = parse_with_authored(&allocator, source);
        assert!(native.unsupported.is_empty());
        assert_eq!(
            cstr!("{:?}", (&native.tree, &native.authored, &native.errors)),
            cstr!("{compat:?}"),
            "{source}"
        );
        assert_eq!(check_fidelity(&native.tree), Ok(()));
        if let Some(authored) = &native.authored {
            assert_eq!(check_fidelity(authored), Ok(()));
        }
        let (normal, errors) = parse(&allocator, source);
        assert_eq!(cstr!("{:?}", native.tree), cstr!("{normal:?}"));
        assert_eq!(cstr!("{:?}", native.errors), cstr!("{errors:?}"));
    }
}

#[test]
fn native_admission_keeps_component_default_delimiters_and_scoped_v_pre() {
    let allocator = Allocator::new();
    let errors = parse_component(&allocator, "<!DOCTYPE html><p>x</p>")
        .unwrap()
        .errors;
    assert!(
        errors
            .iter()
            .any(|error| error.code == ErrorCode::IncorrectlyOpenedComment)
    );
    let tree = parse_component(&allocator, "[[ custom ]] {{{ raw }}}")
        .unwrap()
        .tree;
    let interpolation = tree.children.iter().find_map(|child| match child {
        SurfaceChild::Interpolation(node) => Some(node),
        _ => None,
    });
    let interpolation = interpolation.expect("default double-mustache interpolation");
    assert_eq!(interpolation.open.text, "{{");
    assert_eq!(interpolation.content.text, "{ raw ");
    assert_eq!(interpolation.close.text, "}}");
    assert_eq!(check_fidelity(&tree), Ok(()));

    let tree = parse_component(&allocator, "<div v-pre>{{ expression }}</div>")
        .unwrap()
        .tree;
    let element = tree.children.iter().find_map(|child| match child {
        SurfaceChild::Element(node) => Some(node),
        _ => None,
    });
    let element = element.expect("v-pre element");
    assert!(matches!(
        element.children.first(),
        Some(SurfaceChild::Text(token)) if token.text == "{{ expression }}"
    ));
}
