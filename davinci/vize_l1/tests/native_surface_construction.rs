//! Ordinary-library native construction: structure, recovery and source bytes.

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
    let (native, native_errors) = parse_component_with_options(&allocator, source, options);
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
        let (tree, _) = parse_component(&allocator, fixture.source);
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
        "<section><a><span><a></a><span v-pre>{{ inside }}</span>{{ tail }}</span></a></section>",
    ] {
        let allocator = Allocator::new();
        let native = parse_component_with_authored(&allocator, source);
        let compat = parse_with_authored(&allocator, source);
        assert_eq!(cstr!("{native:?}"), cstr!("{compat:?}"), "{source}");
        assert_eq!(check_fidelity(&native.0), Ok(()));
        if let Some(authored) = &native.1 {
            assert_eq!(check_fidelity(authored), Ok(()));
        }
        let (normal, errors) = parse(&allocator, source);
        assert_eq!(cstr!("{:?}", native.0), cstr!("{normal:?}"));
        assert_eq!(cstr!("{:?}", native.2), cstr!("{errors:?}"));
    }
}

#[test]
fn native_admission_keeps_component_default_delimiters_and_unfinished_v_pre() {
    let allocator = Allocator::new();
    let (_, errors) = parse_component(&allocator, "<!DOCTYPE html><p>x</p>");
    assert!(
        errors
            .iter()
            .any(|error| error.code == ErrorCode::IncorrectlyOpenedComment)
    );
    let (tree, _) = parse_component(&allocator, "[[ custom ]] {{{ raw }}}");
    let interpolation = tree.children.iter().find_map(|child| match child {
        SurfaceChild::Interpolation(node) => Some(node),
        _ => None,
    });
    let interpolation = interpolation.expect("default double-mustache interpolation");
    assert_eq!(interpolation.open.text, "{{");
    assert_eq!(interpolation.content.text, "{ raw ");
    assert_eq!(interpolation.close.text, "}}");
    assert_eq!(check_fidelity(&tree), Ok(()));

    let (tree, _) = parse_component(&allocator, "<div v-pre>{{ expression }}</div>");
    let element = tree.children.iter().find_map(|child| match child {
        SurfaceChild::Element(node) => Some(node),
        _ => None,
    });
    let element = element.expect("v-pre element");
    assert!(matches!(
        element.children.first(),
        Some(SurfaceChild::Interpolation(_))
    ));
}
