//! Parse/lowering diagnostics and span mapping back to source.

mod common;

use common::{lower_all, root_element};
use vize_atelier_jsx::{JsxLang, lower_source};
use vize_carton::Bump;

#[test]
fn valid_source_has_no_diagnostics() {
    let bump = Bump::new();
    let out = lower_all(&bump, "const a = <div/>;");
    assert!(out.diagnostics.is_empty());
    assert!(!out.has_errors());
}

#[test]
fn syntax_error_is_reported_with_a_range() {
    let bump = Bump::new();
    let src = "const a = <div>;";
    let out = lower_source(&bump, src, JsxLang::Jsx);
    assert!(out.has_errors());
    let diag = &out.diagnostics[0];
    assert!(diag.end > diag.start);
    assert!(diag.end as usize <= src.len());
}

#[test]
fn diagnostic_range_maps_into_source() {
    let bump = Bump::new();
    let src = "const a = <div>{</div>;";
    let out = lower_source(&bump, src, JsxLang::Jsx);
    assert!(out.has_errors());
    for diag in &out.diagnostics {
        // Every diagnostic range must be sliceable from the original source.
        let _ = &src[diag.start as usize..diag.end as usize];
    }
}

#[test]
fn element_location_round_trips_through_source() {
    let bump = Bump::new();
    let src = "const App = () => <button class=\"x\">Go</button>;";
    let out = lower_all(&bump, src);
    let element = root_element(&out.roots[0].root);
    let start = element.loc.start.offset as usize;
    let end = element.loc.end.offset as usize;
    assert_eq!(&src[start..end], "<button class=\"x\">Go</button>");
}

#[test]
fn attribute_value_location_round_trips() {
    let bump = Bump::new();
    let src = "const a = <div title=\"hello\"/>;";
    let out = lower_all(&bump, src);
    let attr = match &root_element(&out.roots[0].root).props[0] {
        vize_relief::ast::PropNode::Attribute(a) => a,
        _ => panic!("expected attribute"),
    };
    let value = attr.value.as_ref().unwrap();
    let start = value.loc.start.offset as usize;
    let end = value.loc.end.offset as usize;
    assert_eq!(&src[start..end], "\"hello\"");
}

#[test]
fn line_and_column_are_one_indexed() {
    let bump = Bump::new();
    // `<div/>` begins at column 1 of line 2.
    let src = "x;\n<div/>;";
    let out = lower_all(&bump, src);
    let loc = &root_element(&out.roots[0].root).loc;
    assert_eq!(loc.start.line, 2);
    assert_eq!(loc.start.column, 1);
}
