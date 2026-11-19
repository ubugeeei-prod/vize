use oxc_ast::ast::Expression;
use oxc_parser::ParseOptions;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span};

use super::super::super::{Embed, EmbedHole, EmbedSource, Grammar, Lang, Shape, parse_once};

#[test]
fn borrowed_facade_keeps_the_actual_language_profile_instead_of_inferring_from_source() {
    let arena = Allocator::default();
    let text = "count as number";
    for (lang, hole) in [(Lang::Js, Some(EmbedHole::Syntax)), (Lang::Ts, None)] {
        let grammar = Grammar {
            shape: Shape::Expr,
            lang,
        };
        let original = parse_once(
            &arena,
            Embed {
                source: EmbedSource::authored(text, Span::new(0, 15)).unwrap(),
                grammar,
            },
        );
        assert_eq!(original.hole(), hole);
        let before = arena.allocated_bytes();
        if let Some(view) = original.borrow_expression() {
            assert_eq!(lang, Lang::Ts);
            assert!(core::ptr::eq(view.original(), &original));
            assert_eq!(view.grammar(), grammar);
            assert_eq!(view.source_type(), SourceType::ts().with_module(true));
            assert_eq!(view.admitted_expression().source_type(), view.source_type());
            assert_eq!(view.options(), ParseOptions::default());
            assert!(matches!(view.expression(), Expression::TSAsExpression(_)));
            assert!(core::ptr::eq(view.source().text(), text));
            assert_eq!(
                view.decoded_span(view.admitted_expression().parser_container_span()),
                Ok(Span::new(0, 15))
            );
        } else {
            assert_eq!(lang, Lang::Js);
            assert_eq!(original.source_type(), SourceType::mjs());
            assert!(original.diagnostics().count() > 0);
            assert_eq!(original.source().text(), text);
        }
        assert_eq!(arena.allocated_bytes(), before);
    }
}
