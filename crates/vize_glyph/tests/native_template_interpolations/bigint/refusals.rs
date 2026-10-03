//! Whole-input refusals and unchanged independent admission bounds.

use super::{SCRIPTS, assert_preserved, format, options};
use crate::{operands, selected};
use oxc_ast::ast::{BigintBase, Expression};
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeTemplateRefusal, native_template_document,
};
use vize_l0::Allocator;
use vize_l1::embed::syntax::EmbedHole;

#[test]
fn unsupported_bigint_descendants_or_keys_refuse_the_complete_selected_document() {
    for (content, ts_only) in [
        ("1n,this", false),
        ("1n,f(...a)", false),
        ("[1n,...a]", false),
        ("{1n:a}", false),
        ("{a:1n,b(){} }", false),
        ("1n,(a=b)", false),
        ("1n,a++", false),
        ("1n,await a", false),
        ("1n,new A", false),
        ("1n,a?.b", false),
        ("1n,/x/", false),
        ("1n,`x`", false),
        ("1n,(a as T)", true),
        ("1n,f<T>(a)", true),
        ("1n,a!", true),
    ] {
        for script in SCRIPTS {
            if ts_only && script.is_empty() {
                continue;
            }
            let arena = Allocator::default();
            let source = vize_l0::cstr!(
                "<template>{{{{1n}}}}<p>{{{{/*原*/ {content} }}}}</p></template>{script}"
            );
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let operand = original.get(1).unwrap();
            let syntax = operand.syntax();
            assert_eq!(syntax.hole(), None, "{source}");
            let root = core::ptr::from_ref(syntax.expression().unwrap());
            let view = syntax.source();
            let comments = syntax
                .comments()
                .map(|comment| {
                    (
                        comment.kind(),
                        comment.decoded_span(),
                        comment.authored_span(),
                        comment.text().unwrap().as_ptr(),
                    )
                })
                .collect::<std::vec::Vec<_>>();
            let refs = original.iter().collect::<std::vec::Vec<_>>();
            assert!(
                matches!(
                    native_template_document(&owner, &refs, &arena),
                    Err(NativeTemplateRefusal::Expression {
                        refusal: ExpressionRefusal::UnsupportedNode { .. },
                        ..
                    })
                ),
                "{source}"
            );
            assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), root);
            assert_eq!(
                syntax
                    .comments()
                    .map(|comment| (
                        comment.kind(),
                        comment.decoded_span(),
                        comment.authored_span(),
                        comment.text().unwrap().as_ptr(),
                    ))
                    .collect::<std::vec::Vec<_>>(),
                comments
            );
            assert!(core::ptr::eq(syntax.source().text(), view.text()));
            assert_eq!(syntax.source().span(), view.span());
            assert_eq!(syntax.diagnostics().count(), 0);
            assert!(core::ptr::eq(
                syntax.source().authored_root(),
                source.as_str()
            ));
            assert_eq!(operand.content_span().slice(&source), operand.raw_content());
        }
    }
}

#[test]
fn malformed_bigint_tokens_keep_original_holes_and_complete_source_after_a_supported_operand() {
    for content in [
        "00n", "01n", "1.0n", "1e2n", "1_n", "1__0n", "0x_n", "0b2n", "0o8n", "1N", "1nfoo",
    ] {
        for script in SCRIPTS {
            let arena = Allocator::default();
            let source = vize_l0::cstr!(
                "<template>{{{{1n}}}}<p>{{{{/*keep*/ {content} }}}}</p></template>{script}"
            );
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let operand = original.get(1).unwrap();
            let syntax = operand.syntax();
            assert!(syntax.hole().is_some(), "{source}");
            let hole = syntax.hole();
            let diagnostics = syntax.diagnostics().count();
            let comments = syntax.comments().count();
            let view = syntax.source();
            let refs = original.iter().collect::<std::vec::Vec<_>>();
            assert!(
                matches!(
                    native_template_document(&owner, &refs, &arena),
                    Err(NativeTemplateRefusal::OperandRejected {
                        index: 1,
                        hole: Some(_),
                        ..
                    })
                ),
                "{source}"
            );
            assert_eq!(syntax.hole(), hole);
            assert_eq!(syntax.diagnostics().count(), diagnostics);
            assert_eq!(syntax.comments().count(), comments);
            assert!(syntax.expression().is_none());
            assert!(core::ptr::eq(syntax.source().text(), view.text()));
            assert_eq!(syntax.source().span(), view.span());
            assert!(core::ptr::eq(
                syntax.source().authored_root(),
                source.as_str()
            ));
            assert_eq!(operand.content_span().slice(&source), operand.raw_content());
        }
    }
}

#[test]
fn bigint_non_ascii_gaps_have_no_plain_whitespace_formatting_authority() {
    for content in [
        "1n\u{a0}+2n",
        "1n+\u{a0}2n",
        "1n,\u{a0}2n",
        "f(\u{a0}1n)",
        "{a:\u{a0}1n}",
        "1n+&#160;2n",
    ] {
        for script in SCRIPTS {
            let arena = Allocator::default();
            let source = vize_l0::cstr!("<template>{{{{{content} }}}}</template>{script}");
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let operand = original.first().unwrap();
            let syntax = operand.syntax();
            assert_eq!(syntax.hole(), None, "{source}");
            let root = core::ptr::from_ref(syntax.expression().unwrap());
            let map = syntax
                .source()
                .decode_map()
                .map(|map| map.segments().as_ptr());
            let refs = original.iter().collect::<std::vec::Vec<_>>();
            assert!(
                matches!(
                    native_template_document(&owner, &refs, &arena),
                    Err(NativeTemplateRefusal::Expression {
                        refusal: ExpressionRefusal::InvalidGap { .. },
                        ..
                    })
                ),
                "{source}"
            );
            assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), root);
            assert_eq!(
                syntax
                    .source()
                    .decode_map()
                    .map(|map| map.segments().as_ptr()),
                map
            );
            assert_eq!(operand.content_span().slice(&source), operand.raw_content());
        }
    }
}

#[test]
fn bigint_operands_cannot_be_foreign_missing_extra_reversed_or_duplicated() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!("<template>{{{{1n}}}}<p>{{{{0x2An}}}}</p></template>{script}");
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let a = original.first().unwrap();
        let b = original.get(1).unwrap();
        let roots = [
            core::ptr::from_ref(a.syntax().expression().unwrap()),
            core::ptr::from_ref(b.syntax().expression().unwrap()),
        ];
        assert!(native_template_document(&owner, &[a, b], &arena).is_ok());
        assert!(matches!(
            native_template_document(&owner, &[], &arena),
            Err(NativeTemplateRefusal::MissingOperand { index: 0, .. })
        ));
        assert!(matches!(
            native_template_document(&owner, &[a], &arena),
            Err(NativeTemplateRefusal::MissingOperand { index: 1, .. })
        ));
        assert!(matches!(
            native_template_document(&owner, &[a, b, a], &arena),
            Err(NativeTemplateRefusal::ExtraOperand { index: 2 })
        ));
        for (refs, index) in [([b, a], 0), ([a, a], 1)] {
            assert!(matches!(native_template_document(&owner, &refs, &arena),
                Err(NativeTemplateRefusal::OperandRejected { index: rejected, .. }) if rejected == index));
        }
        let copied = source.to_owned();
        for foreign in [selected(&arena, &source), selected(&arena, &copied)] {
            assert!(matches!(
                native_template_document(&foreign, &[a, b], &arena),
                Err(NativeTemplateRefusal::OperandRejected { index: 0, .. })
            ));
        }
        assert_eq!(
            [
                core::ptr::from_ref(a.syntax().expression().unwrap()),
                core::ptr::from_ref(b.syntax().expression().unwrap())
            ],
            roots
        );
        assert_eq!(a.raw_content(), "1n");
        assert_eq!(b.raw_content(), "0x2An");
    }
}

#[test]
fn bigint_leaf_depth_and_whole_wrapper_unit_budgets_remain_independent_and_immutable() {
    let supported = vize_l0::cstr!("{}1n", "!".repeat(16));
    let refused = vize_l0::cstr!("{}1n", "!".repeat(17));
    let sequence = std::iter::repeat_n("1n", 15)
        .collect::<std::vec::Vec<_>>()
        .join(",");
    for script in SCRIPTS {
        let source = vize_l0::cstr!("<template>{{{{{supported}}}}}</template>{script}");
        assert_eq!(
            format(&source, options(200, LineEnding::Lf)),
            vize_l0::cstr!("{{{{ {}1n }}}}", "! ".repeat(16))
        );
        let arena = Allocator::default();
        let source = vize_l0::cstr!("<template>{{{{{refused}}}}}</template>{script}");
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let operand = original.first().unwrap();
        let syntax = operand.syntax();
        assert_eq!(syntax.hole(), None);
        let root = core::ptr::from_ref(syntax.expression().unwrap());
        let mut leaf = syntax.expression().unwrap();
        for _ in 0..17 {
            let Expression::UnaryExpression(unary) = leaf else {
                panic!("original unary depth")
            };
            leaf = &unary.argument;
        }
        let Expression::BigIntLiteral(literal) = leaf else {
            panic!("genuine BigInt leaf")
        };
        assert_eq!(
            (
                literal.base,
                literal.value.as_str(),
                literal.raw.unwrap().as_str()
            ),
            (BigintBase::Decimal, "1", "1n")
        );
        let refs = original.iter().collect::<std::vec::Vec<_>>();
        let Err(NativeTemplateRefusal::Expression {
            refusal: ExpressionRefusal::DepthLimit { span },
            ..
        }) = native_template_document(&owner, &refs, &arena)
        else {
            panic!("depth17")
        };
        assert_eq!(span.slice(syntax.source().text()), "1n");
        assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), root);
        assert_eq!(operand.content_span().slice(&source), operand.raw_content());
        for (content, hole) in [
            (sequence.clone(), None),
            (
                vize_l0::cstr!("!{sequence}").to_string(),
                Some(EmbedHole::TokenBudget),
            ),
        ] {
            let source = vize_l0::cstr!("<template>{{{{{content}}}}}</template>{script}");
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let operand = original.first().unwrap();
            let syntax = operand.syntax();
            assert_eq!(
                syntax.hole(),
                hole,
                "29 original units plus two wrapper units are the fixed limit"
            );
            let refs = original.iter().collect::<std::vec::Vec<_>>();
            if hole.is_none() {
                let Expression::SequenceExpression(sequence) = syntax.expression().unwrap() else {
                    panic!("Sequence")
                };
                assert_eq!(sequence.expressions.len(), 15);
                assert!(
                    sequence
                        .expressions
                        .iter()
                        .all(|item| matches!(item, Expression::BigIntLiteral(_)))
                );
                assert!(native_template_document(&owner, &refs, &arena).is_ok());
            } else {
                assert!(syntax.expression().is_none());
                assert_eq!(syntax.diagnostics().count(), 0);
                assert_eq!(syntax.comments().count(), 0);
                assert!(matches!(
                    native_template_document(&owner, &refs, &arena),
                    Err(NativeTemplateRefusal::OperandRejected {
                        index: 0,
                        hole: Some(EmbedHole::TokenBudget),
                        ..
                    })
                ));
            }
            assert_eq!(operand.content_span().slice(&source), operand.raw_content());
            assert!(core::ptr::eq(
                syntax.source().authored_root(),
                source.as_str()
            ));
        }
    }
    assert_preserved(&supported);
    assert_preserved(&sequence);
}
