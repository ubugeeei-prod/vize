//! Checked projection and custody controls over actual original L1 syntax.

use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::{
    Embed, EmbedSource, Grammar, Lang, Shape, SourceError, prepare_attribute_value,
    syntax::{RetainedExpression, parse_once},
};

use super::{ExpressionRefusal, expression_document, source::Context};
use crate::native_doc::document::Kind;

fn retained<'a>(allocator: &'a Allocator, source: EmbedSource<'a>) -> RetainedExpression<'a> {
    parse_once(
        allocator,
        Embed {
            source,
            grammar: Grammar {
                shape: Shape::Expr,
                lang: Lang::Js,
            },
        },
    )
    .into_expression()
    .unwrap()
}

#[test]
fn atom_text_and_consumed_parts_borrow_the_exact_checked_original() {
    let allocator = Allocator::default();
    let root = "é name tail";
    let original = retained(
        &allocator,
        EmbedSource::authored(root, Span::new(3, 7)).unwrap(),
    );
    let block = SourceRoot::new(root).unwrap().whole_block();
    let (same, document) = expression_document(&original, block, &allocator)
        .unwrap()
        .into_parts();
    assert!(core::ptr::eq(same, &original));
    let Kind::Concat(parts) = document.kind else {
        panic!("whole document")
    };
    let Kind::Text(text) = parts[1].kind else {
        panic!("atom")
    };
    assert_eq!(text, "name");
    assert_eq!(text.as_ptr(), root.get(3..7).unwrap().as_ptr());
}

#[test]
fn exact_projection_refuses_decode_interiors_and_utf8_splits() {
    let allocator = Allocator::default();
    for (root, decode, expected) in [
        ("&fjlig;", true, SourceError::PartialEntityBoundary),
        ("日本", false, SourceError::InvalidDecodedSpan),
    ] {
        let source = if decode {
            prepare_attribute_value(&allocator, root, Span::new(0, root.len() as u32)).unwrap()
        } else {
            EmbedSource::authored(root, Span::new(0, root.len() as u32)).unwrap()
        };
        let original = retained(&allocator, source);
        let context = Context::new(
            &original,
            SourceRoot::new(root).unwrap().whole_block(),
            &allocator,
        )
        .unwrap();
        assert_eq!(
            context.authored_span(Span::new(0, 1)),
            Err(ExpressionRefusal::Projection {
                offset: 0,
                error: expected
            })
        );
        assert_eq!(
            context.authored_span(Span::new(0, source.text().len() as u32)),
            Ok(source.span())
        );
    }
}

#[test]
fn unknown_gap_bytes_and_child_ranges_cannot_establish_token_framing() {
    let allocator = Allocator::default();
    let root = "a+b";
    let original = retained(
        &allocator,
        EmbedSource::authored(root, Span::new(0, 3)).unwrap(),
    );
    let mut context = Context::new(
        &original,
        SourceRoot::new(root).unwrap().whole_block(),
        &allocator,
    )
    .unwrap();
    assert_eq!(
        context.token(Span::new(1, 2), "-"),
        Err(ExpressionRefusal::InvalidGap {
            span: Span::new(1, 2)
        })
    );
    assert!(matches!(
        context.node(original.expression().unwrap(), Span::new(0, 1), 0),
        Err(ExpressionRefusal::InvalidFraming { .. })
    ));
    assert_eq!(original.source().text(), root);
}
