use super::{Cx, ExprRef, OpaqueReason, expression};
use crate::dialect::LegacyCaps;
use vize_l0::{Allocator, SourceRoot, Span, String};

#[test]
fn detached_and_inconsistent_encoded_sources_have_an_explicit_coordinate_refusal() {
    let a = Allocator::default();
    let source = "'&amp;' && value";
    let cx = Cx::with_source_block_and_comment_policy(
        &a,
        SourceRoot::new(source).unwrap().whole_block(),
        LegacyCaps::VUE3,
        true,
        &[],
        None,
    );
    let detached = String::new(source);
    let whole = Span::new(0, source.len() as u32);
    for (raw, span) in [
        (detached.as_str(), whole),
        (source, Span::new(0, whole.end - 1)),
        (source, Span::new(0, whole.end + 1)),
    ] {
        let ExprRef::Opaque(opaque) = expression(&cx, raw, span, false) else {
            panic!("no AST from invalid source custody");
        };
        assert_eq!(opaque.reason, OpaqueReason::SourceCoordinates);
        assert_eq!(opaque.source, raw);
        assert_eq!(opaque.span, span);
    }
}

#[test]
fn decoded_parse_failures_keep_the_existing_parser_classification_and_text() {
    let a = Allocator::default();
    let source = "a &amp;&amp; broken +";
    let cx = Cx::with_source_block_and_comment_policy(
        &a,
        SourceRoot::new(source).unwrap().whole_block(),
        LegacyCaps::VUE3,
        true,
        &[],
        None,
    );
    let whole = Span::new(0, source.len() as u32);
    let ExprRef::Opaque(opaque) = expression(&cx, source, whole, false) else {
        panic!("invalid expression remains opaque");
    };
    assert_eq!(opaque.reason, OpaqueReason::ParseRejected);
    assert_eq!(opaque.source, "a && broken +");
    assert_eq!(opaque.span, whole);
}
