//! Parse/lowering diagnostics and span mapping back to source.

mod common;

use common::{lower_all, root_element};
use vize_atelier_jsx::{JsxLang, lower_source};
use vize_s0::Allocator;

#[test]
fn valid_source_has_no_diagnostics() {
    let bump = Allocator::new();
    let out = lower_all(&bump, "const a = <div/>;");
    assert!(out.diagnostics.is_empty());
    assert!(!out.has_errors());
}

#[test]
fn syntax_error_is_reported_with_a_range() {
    let bump = Allocator::new();
    let src = "const a = <div>;";
    let out = lower_source(&bump, bump.as_oxc(), src, JsxLang::Jsx);
    assert!(out.has_errors());
    let diag = &out.diagnostics[0];
    assert!(diag.end > diag.start);
    assert!(diag.end as usize <= src.len());
}

#[test]
fn diagnostic_range_maps_into_source() {
    let bump = Allocator::new();
    let src = "const a = <div>{</div>;";
    let out = lower_source(&bump, bump.as_oxc(), src, JsxLang::Jsx);
    assert!(out.has_errors());
    for diag in &out.diagnostics {
        // Every diagnostic range must be sliceable from the original source.
        let _ = &src[diag.start as usize..diag.end as usize];
    }
}

#[test]
fn element_location_round_trips_through_source() {
    let bump = Allocator::new();
    let src = "const App = () => <button class=\"x\">Go</button>;";
    let out = lower_all(&bump, src);
    let element = root_element(&out.roots[0].root);
    let start = element.loc.span.start as usize;
    let end = element.loc.span.end as usize;
    assert_eq!(&src[start..end], "<button class=\"x\">Go</button>");
}

#[test]
fn attribute_value_location_round_trips() {
    let bump = Allocator::new();
    let src = "const a = <div title=\"hello\"/>;";
    let out = lower_all(&bump, src);
    let attr = match &root_element(&out.roots[0].root).props[0] {
        vize_relief::PropNode::Attribute(a) => a,
        _ => panic!("expected attribute"),
    };
    let value = attr.value.as_ref().unwrap();
    let start = value.loc.span.start as usize;
    let end = value.loc.span.end as usize;
    assert_eq!(&src[start..end], "\"hello\"");
}

#[test]
fn line_and_column_derive_from_the_span_offset() {
    let bump = Allocator::new();
    // `<div/>` begins at column 1 of line 2 (0-indexed: line 1, column 0).
    // Nodes store byte offsets only; line/column are derived at the edges
    // that render them, via `vize_s0::line_index` (Davinci P1-4).
    let src = "x;\n<div/>;";
    let out = lower_all(&bump, src);
    let loc = &root_element(&out.roots[0].root).loc;
    let (line, column) = vize_s0::line_index::offset_to_line_col(src, loc.span.start as usize);
    assert_eq!(line, 1);
    assert_eq!(column, 0);
}

/// The shapes below all used to lower to something meaningless with no signal
/// at all: the component rendered wrong and nothing said so (#3421).
fn diagnostic_texts<'d>(
    out: &'d vize_atelier_jsx::LowerOutput<'_>,
) -> std::vec::Vec<(bool, &'d str)> {
    out.diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.is_error(), diagnostic.message.as_str()))
        .collect()
}

/// `<a:b/>` names a namespace nothing downstream resolves, and the tag used to
/// reach codegen verbatim as `createElementBlock("a:b")` with no signal.
/// `@vue/babel-plugin-jsx` rejects every namespaced tag outright; Vize keeps the
/// two prefixes that name a real element namespace and rejects the rest (#3421).
#[test]
fn unsupported_tag_namespace_is_reported() {
    let bump = Allocator::new();
    let src = "const a = <a:b foo={1}/>;";
    let out = lower_all(&bump, src);

    assert_eq!(
        diagnostic_texts(&out),
        vec![(
            true,
            "unsupported JSX tag namespace `a:`; only `svg:` and `math:` name a real element \
             namespace, so `a:b` would be emitted verbatim as a tag name nothing resolves \
             (`@vue/babel-plugin-jsx` rejects every namespaced tag)"
        )]
    );
    // The span covers exactly the tag name.
    let diagnostic = &out.diagnostics[0];
    assert_eq!(
        &src[diagnostic.start as usize..diagnostic.end as usize],
        "a:b"
    );
}

#[test]
fn svg_and_math_namespaced_tags_are_not_reported() {
    for source in [
        "const a = <svg:circle/>;",
        "const a = <math:mi/>;",
        "const a = <div><svg:circle/></div>;",
    ] {
        let bump = Allocator::new();
        let out = lower_all(&bump, source);
        assert_eq!(diagnostic_texts(&out), vec![], "for {source}");
    }
}

/// A fragment is lowered faithfully wherever it appears, so none of these
/// positions reports. The nested case used to report because it produced an
/// unresolvable `Fragment` component; it is now spliced into its parent.
#[test]
fn fragments_are_not_reported_at_any_depth() {
    for source in [
        "const a = <><i/><b/></>;",
        "const a = <div><><i/></></div>;",
        "const a = <B>{() => <><i/></>}</B>;",
        "const a = <div>{cond ? <><i/></> : <b/>}</div>;",
    ] {
        let bump = Allocator::new();
        let out = lower_all(&bump, source);
        assert_eq!(diagnostic_texts(&out), vec![], "for {source}");
    }
}

#[test]
fn spread_child_is_reported() {
    let bump = Allocator::new();
    let src = "const a = <div>{...items}</div>;";
    let out = lower_all(&bump, src);

    assert_eq!(
        diagnostic_texts(&out),
        vec![(
            true,
            "spread children (`{...items}`) are not supported; the value would be stringified instead of spread"
        )]
    );
    let diagnostic = &out.diagnostics[0];
    assert_eq!(
        &src[diagnostic.start as usize..diagnostic.end as usize],
        "{...items}"
    );
}
