//! Original parser metadata and immutable BigInt storage custody.

use super::{BigIntFingerprint, SCRIPTS, WIDTHS, assert_preserved, format, newline, options};
use crate::{operands, preservation::fingerprint, selected};
use oxc_ast::ast::{BigintBase, Expression};
use vize_glyph::native_doc::{LineEnding, NativeTemplateRefusal, native_template_document, print};
use vize_l0::Allocator;
use vize_l1::embed::syntax::EmbedHole;

#[test]
fn bigint_base_value_raw_option_and_authored_spelling_remain_independent_on_reparse() {
    // Values below are original parser Str metadata, not JavaScript arithmetic results.
    for (content, base, value, raw) in [
        ("42n", BigintBase::Decimal, "42", "42n"),
        ("0b10_1010n", BigintBase::Binary, "42", "0b10_1010n"),
        ("0o5_2n", BigintBase::Octal, "42", "0o5_2n"),
        ("0x2_An", BigintBase::Hex, "42", "0x2_An"),
        ("1_000n", BigintBase::Decimal, "1000", "1_000n"),
        ("&#48;x2_A&#110;", BigintBase::Hex, "42", "0x2_An"),
        (
            "9007199254740993n",
            BigintBase::Decimal,
            "9007199254740993",
            "9007199254740993n",
        ),
    ] {
        for script in SCRIPTS {
            let arena = Allocator::default();
            let source = vize_l0::cstr!("<template>{{{{{content}}}}}</template>{script}");
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let syntax = original.first().unwrap().syntax();
            assert_eq!(syntax.hole(), None);
            let Expression::BigIntLiteral(literal) = syntax.expression().unwrap() else {
                panic!("actual original BigInt")
            };
            assert_eq!(literal.base, base);
            assert_eq!(literal.value.as_str(), value);
            assert_eq!(literal.raw.map(|raw| raw.as_str()), Some(raw));
            let authored = syntax.authored_span(literal.span).unwrap().slice(&source);
            assert_eq!(authored, content);
            let expected = BigIntFingerprint::from_literal(literal, authored.to_owned());
            for width in WIDTHS {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let output = format(&source, options(width, ending));
                    let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                    let second = selected(&arena, &replay);
                    let second_operands = operands(&second);
                    let same = second_operands.first().unwrap().syntax();
                    assert_eq!(same.hole(), None);
                    let Expression::BigIntLiteral(literal) = same.expression().unwrap() else {
                        panic!("reparsed BigInt")
                    };
                    assert_eq!(
                        BigIntFingerprint::from_literal(
                            literal,
                            same.authored_span(literal.span)
                                .unwrap()
                                .slice(&replay)
                                .to_owned()
                        ),
                        expected
                    );
                    assert_eq!(format(&replay, options(width, ending)), output);
                }
            }
        }
        assert_preserved(content);
    }
    for (left, right) in [
        ("1n", "1"),
        ("42n", "0x2An"),
        ("0x2an", "0X2An"),
        ("1_000n", "1000n"),
        ("1n", "&#49;&#110;"),
        ("{a:1n}", "{a:1}"),
        ("f(1n)", "f(1)"),
    ] {
        for script in SCRIPTS {
            let arena = Allocator::default();
            let a_source = vize_l0::cstr!("<template>{{{{{left} }}}}</template>{script}");
            let b_source = vize_l0::cstr!("<template>{{{{{right} }}}}</template>{script}");
            let a_owner = selected(&arena, &a_source);
            let b_owner = selected(&arena, &b_source);
            let a = operands(&a_owner);
            let b = operands(&b_owner);
            let a = a.first().unwrap().syntax();
            let b = b.first().unwrap().syntax();
            assert_eq!(a.hole(), None);
            assert_eq!(b.hole(), None);
            assert_ne!(
                fingerprint(a, a.expression().unwrap()),
                fingerprint(b, b.expression().unwrap())
            );
        }
        assert_preserved(&vize_l0::cstr!("{left} "));
        assert_preserved(&vize_l0::cstr!("{right} "));
    }
}

#[test]
fn bigint_document_borrows_original_literal_call_comment_map_and_content_storage() {
    let content = "/*#__PURE__*/f&#40;/*x*/0x2_A&#110;&#41;";
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source =
            vize_l0::cstr!("<!--前--><template><p>{{{{ \t{content} \t}}}}</p></template>{script}");
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let operand = original.first().unwrap();
        let syntax = operand.syntax();
        assert_eq!(syntax.hole(), None);
        assert!(syntax.source_type().is_module());
        assert_eq!(syntax.source_type().is_typescript(), !script.is_empty());
        let ast = core::ptr::from_ref(syntax.expression().unwrap());
        let Expression::CallExpression(call) = syntax.expression().unwrap() else {
            panic!("pure Call")
        };
        assert!(call.pure && !call.optional && call.type_arguments.is_none());
        let call_root = core::ptr::from_ref(&**call);
        let callee = core::ptr::from_ref(&call.callee);
        let arguments = call.arguments.as_ptr();
        let argument = call.arguments.first().unwrap().as_expression().unwrap();
        let argument_root = core::ptr::from_ref(argument);
        let Expression::BigIntLiteral(literal) = argument else {
            panic!("original BigInt argument")
        };
        let literal_root = core::ptr::from_ref(&**literal);
        let span = literal.span;
        let value = literal.value.as_str();
        let raw = literal.raw.unwrap().as_str();
        assert_eq!(
            (literal.base, value, raw),
            (BigintBase::Hex, "42", "0x2_An")
        );
        let authored = syntax.authored_span(span).unwrap();
        assert_eq!(authored.slice(&source), "0x2_A&#110;");
        let comments = syntax
            .comments()
            .map(|comment| {
                (
                    comment.kind(),
                    comment.decoded_span().unwrap(),
                    comment.authored_span().unwrap(),
                    comment.text().unwrap().as_ptr(),
                    comment.authored_span().unwrap().slice(&source).to_owned(),
                )
            })
            .collect::<std::vec::Vec<_>>();
        assert_eq!(comments.len(), 2);
        let view = syntax.source();
        let map = view.decode_map().unwrap().segments();
        let map_values = map.to_vec();
        let content_pointer = operand.raw_content().as_ptr();
        let content_span = operand.content_span();
        let full = operand.full_span();
        assert!(view.span().start > 0);
        assert_eq!(
            content_span.slice(&source),
            vize_l0::cstr!(" \t{content} \t")
        );
        let refs = original.iter().collect::<std::vec::Vec<_>>();
        let document = native_template_document(&owner, &refs, &arena).unwrap();
        for width in WIDTHS {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                let generated = newline(ending);
                let body = "/*#__PURE__*/f &#40;/*x*/0x2_A&#110; &#41;";
                let expected = if width < 80 {
                    vize_l0::cstr!("<p>{{{{{generated}    {body}{generated}  }}}}</p>")
                } else {
                    vize_l0::cstr!("<p>{{{{ {body} }}}}</p>")
                };
                let output = print(document.document(), &options(width, ending));
                assert_eq!(output, expected, "{source} / {width} / {ending:?}");
                assert!(core::ptr::eq(document.original(), &owner));
                assert!(core::ptr::eq(document.operands(), refs.as_slice()));
                assert!(core::ptr::eq(
                    *document.operands().first().unwrap(),
                    operand
                ));
                assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), ast);
                assert_eq!(core::ptr::from_ref(&**call), call_root);
                assert!(call.pure && !call.optional && call.type_arguments.is_none());
                assert_eq!(core::ptr::from_ref(&call.callee), callee);
                assert_eq!(call.arguments.as_ptr(), arguments);
                assert_eq!(call.arguments.len(), 1);
                assert_eq!(
                    core::ptr::from_ref(call.arguments.first().unwrap().as_expression().unwrap()),
                    argument_root
                );
                assert_eq!(core::ptr::from_ref(&**literal), literal_root);
                assert_eq!(literal.span, span);
                assert_eq!(literal.base, BigintBase::Hex);
                assert!(core::ptr::eq(literal.value.as_str(), value));
                assert!(core::ptr::eq(literal.raw.unwrap().as_str(), raw));
                assert_eq!(syntax.authored_span(literal.span), Ok(authored));
                assert_eq!(
                    syntax
                        .comments()
                        .map(|comment| (
                            comment.kind(),
                            comment.decoded_span().unwrap(),
                            comment.authored_span().unwrap(),
                            comment.text().unwrap().as_ptr(),
                            comment.authored_span().unwrap().slice(&source).to_owned(),
                        ))
                        .collect::<std::vec::Vec<_>>(),
                    comments
                );
                assert!(core::ptr::eq(
                    syntax.source().decode_map().unwrap().segments(),
                    map
                ));
                assert_eq!(
                    syntax.source().decode_map().unwrap().segments(),
                    map_values.as_slice()
                );
                assert!(core::ptr::eq(
                    syntax.source().authored_root(),
                    source.as_str()
                ));
                assert!(core::ptr::eq(syntax.source().text(), view.text()));
                assert_eq!(syntax.source().span(), view.span());
                assert_eq!(operand.raw_content().as_ptr(), content_pointer);
                assert_eq!(operand.content_span(), content_span);
                assert_eq!(operand.full_span(), full);
                assert_eq!(content_span.slice(&source), operand.raw_content());
                let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                assert_eq!(format(&replay, options(width, ending)), output);
            }
        }
    }
    assert_preserved(content);
}

#[test]
fn genuine_bare_bigint_numeric_run_includes_the_suffix_at_the_4096_byte_guard() {
    let digits = "9".repeat(4095);
    let literal = vize_l0::cstr!("{digits}n");
    assert_eq!(literal.len(), 4096);
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!("<template><p>{{{{{literal}}}}}</p></template>{script}");
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let syntax = original.first().unwrap().syntax();
        assert_eq!(syntax.hole(), None);
        let Expression::BigIntLiteral(node) = syntax.expression().unwrap() else {
            panic!("bare BigInt")
        };
        assert_eq!(node.value.as_str(), digits);
        assert_eq!(node.raw.unwrap().as_str(), literal);
        assert_eq!(node.base, BigintBase::Decimal);
        let root = core::ptr::from_ref(&**node);
        for width in WIDTHS {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                let generated = newline(ending);
                let output = format(&source, options(width, ending));
                assert_eq!(
                    output,
                    vize_l0::cstr!("<p>{{{{{generated}    {literal}{generated}  }}}}</p>")
                );
                let replay = vize_l0::cstr!("<template>{output}</template>{script}");
                assert_eq!(format(&replay, options(width, ending)), output);
                assert_eq!(core::ptr::from_ref(&**node), root);
            }
        }
        let too_large = vize_l0::cstr!("{}n", "9".repeat(4096));
        assert_eq!(too_large.len(), 4097);
        let source = vize_l0::cstr!("<template><p>{{{{{too_large}}}}}</p></template>{script}");
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let operand = original.first().unwrap();
        let syntax = operand.syntax();
        assert_eq!(syntax.hole(), Some(EmbedHole::SafetyAdmission));
        assert!(syntax.expression().is_none());
        assert_eq!(syntax.diagnostics().count(), 0);
        assert_eq!(syntax.comments().count(), 0);
        let refs = original.iter().collect::<std::vec::Vec<_>>();
        assert!(matches!(
            native_template_document(&owner, &refs, &arena),
            Err(NativeTemplateRefusal::OperandRejected {
                index: 0,
                hole: Some(EmbedHole::SafetyAdmission),
                ..
            })
        ));
        assert_eq!(operand.raw_content(), too_large);
        assert_eq!(operand.content_span().slice(&source), too_large);
        assert!(core::ptr::eq(
            syntax.source().authored_root(),
            source.as_str()
        ));
    }
    assert_preserved(&literal);
}
