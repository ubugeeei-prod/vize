//! Genuine private borrowed-source controls; no historical comment admission.

use super::super::{ExpressionRefusal, borrowed_document, source::Context};
use super::Origin;
use crate::native_doc::{PrintOptions, print};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::dialect::vue2::surface;
use vize_l1::embed::syntax::parse_once;
use vize_l1::embed::{Embed, Grammar, Lang, Shape, SourceError, prepare_attribute_value};

#[test]
fn genuine_private_borrowed_comments_and_entity_projection_keep_stock_authority() {
    let arena = Allocator::default();
    for text in ["a/*keep*/+b", "&fjlig;"] {
        let source =
            prepare_attribute_value(&arena, text, Span::new(0, text.len() as u32)).unwrap();
        let owner = parse_once(
            &arena,
            Embed {
                source,
                grammar: Grammar {
                    shape: Shape::Expr,
                    lang: Lang::Js,
                },
            },
        );
        let view = owner.borrow_expression().unwrap();
        let root = view.expression() as *const _;
        let before: std::vec::Vec<_> = view
            .comments()
            .map(|c| (c.decoded_span().unwrap(), c.text().unwrap().as_ptr()))
            .collect();
        let block = SourceRoot::new(text).unwrap().whole_block();
        let document = borrowed_document(&view, block, &arena).unwrap();
        assert_eq!(
            print(&document, &PrintOptions::default()),
            if text == "&fjlig;" {
                "&fjlig;"
            } else {
                "a/*keep*/+ b"
            }
        );
        assert_eq!(view.expression() as *const _, root);
        assert_eq!(
            view.comments()
                .map(|c| (c.decoded_span().unwrap(), c.text().unwrap().as_ptr()))
                .collect::<std::vec::Vec<_>>(),
            before
        );
        let context = Context::from_origin(Origin::Borrowed(&view), block, &arena).unwrap();
        if text == "&fjlig;" {
            assert_eq!(
                context.authored_span(Span::new(0, 1)),
                Err(ExpressionRefusal::Projection {
                    offset: 0,
                    error: SourceError::PartialEntityBoundary
                })
            );
            assert_eq!(context.authored_span(Span::new(0, 2)), Ok(Span::new(0, 7)));
        }
        let copied = text.to_owned();
        assert!(matches!(
            borrowed_document(
                &view,
                SourceRoot::new(&copied).unwrap().whole_block(),
                &arena
            ),
            Err(ExpressionRefusal::SourceMismatch { .. })
        ));
    }
    // Native Vue2 does not grant the stock comment case a TextView.
    let owner = surface::parse_component(&arena, "{{ a/*keep*/+b }}").unwrap();
    assert_eq!(
        owner
            .text_for(owner.children().next().unwrap())
            .unwrap_err(),
        surface::TextRefusal::Boundary(
            vize_l1::dialect::vue2::text::TextBoundaryKind::CommentSyntax
        )
    );
}
