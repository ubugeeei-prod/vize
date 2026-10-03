//! Authored LF survives mapped identity gaps without classifying raw comments.

use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_glyph::native_doc::{LineEnding, PrintOptions, expression_document, print};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::{DecodeSegmentKind, Lang, syntax::RetainedExpression};

use super::{format, retained};

fn options(width: usize, line_ending: LineEnding) -> PrintOptions {
    PrintOptions {
        width,
        line_ending,
        ..PrintOptions::default()
    }
}

#[test]
fn mapped_parentheses_and_unary_gaps_keep_complete_original_physical_lf_or_crlf() {
    for source in [
        "(&#39;//x&#39;\n)",
        "( \n&#39;//x&#39;\r\n )",
        "!\n(&#39;//x&#39;\n)",
        "(&fjlig;\n)",
        "(\n&#39;x&#39;)",
        "&#39;//x&#39; +\r\n1",
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 7, 200] {
                for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(
                        format(source, lang, true, options(width, line_ending)),
                        source
                    );
                }
            }
        }
    }
}

#[test]
fn mapped_binary_logical_gaps_keep_authored_lf_separate_from_generated_line_endings() {
    for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
        let generated = if line_ending == LineEnding::Lf {
            "\n"
        } else {
            "\r\n"
        };
        for (source, expected) in [
            (
                "&#39;//x&#39;\n+1",
                std::format!("&#39;//x&#39;\n+{generated}  1"),
            ),
            (
                "a\r\n+ tr&#117;e",
                std::format!("a\r\n+{generated}  tr&#117;e"),
            ),
            (
                "a\n&& tr&#117;e",
                std::format!("a\n&&{generated}  tr&#117;e"),
            ),
        ] {
            for lang in [Lang::Js, Lang::Ts] {
                for width in [0, 7, 200] {
                    assert_eq!(
                        format(source, lang, true, options(width, line_ending)),
                        expected
                    );
                }
            }
        }
    }
}

#[test]
fn identity_preparation_and_plain_sources_still_normalize_proven_ascii_gaps() {
    let source = "(\n a\n+\n b\n)";
    for lang in [Lang::Js, Lang::Ts] {
        for decode in [false, true] {
            let allocator = Allocator::default();
            let original = retained(
                &allocator,
                source,
                Span::new(0, source.len() as u32),
                lang,
                decode,
            );
            assert!(original.source().decode_map().is_none());
            assert_eq!(original.source().text().as_ptr(), source.as_ptr());
            for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                let generated = if line_ending == LineEnding::Lf {
                    "\n"
                } else {
                    "\r\n"
                };
                for width in [7, 200] {
                    assert_eq!(
                        format(source, lang, decode, options(width, line_ending)),
                        "(a + b)"
                    );
                }
                assert_eq!(
                    format(source, lang, decode, options(0, line_ending)),
                    std::format!("(a +{generated}  b)")
                );
            }
        }
        for width in [0, 7, 200] {
            for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                assert_eq!(
                    format("( &#39;x&#39; )", lang, true, options(width, line_ending)),
                    "(&#39;x&#39;)"
                );
            }
        }
    }
}

#[test]
fn mapped_identity_gap_preserves_same_original_ast_source_and_decode_map_storage() {
    let allocator = Allocator::default();
    let root = "é (&#39;//x&#39;\r\n ) tail";
    let selected = "(&#39;//x&#39;\r\n )";
    let span = Span::new(3, (3 + selected.len()) as u32);
    let original = retained(&allocator, root, span, Lang::Ts, true);
    let Expression::ParenthesizedExpression(parentheses) = original.expression().unwrap() else {
        panic!("original parentheses")
    };
    let Expression::StringLiteral(string) = &parentheses.expression else {
        panic!("actual decoded string")
    };
    assert_eq!(string.value.as_str(), "//x");
    assert_eq!(original.comments().count(), 0);
    let inner = original
        .decoded_span(parentheses.expression.span())
        .unwrap();
    let entire = original.decoded_span(parentheses.span()).unwrap();
    let gap = original
        .source()
        .authored_span(Span::new(inner.end, entire.end - 1))
        .unwrap();
    assert_eq!(
        root.get(gap.start as usize..gap.end as usize),
        Some("\r\n ")
    );
    let map = original.source().decode_map().unwrap();
    assert!(map.segments().iter().any(|segment| {
        segment.kind() == DecodeSegmentKind::Identity
            && segment.authored().start <= gap.start
            && segment.authored().end >= gap.end
    }));
    let ast = core::ptr::from_ref(original.expression().unwrap());
    let child = core::ptr::from_ref(&parentheses.expression);
    let decoded = original.source().text().as_ptr();
    let segments = map.segments().as_ptr();
    let block = SourceRoot::new(root)
        .unwrap()
        .block(root.get(3..span.end as usize).unwrap(), 3)
        .unwrap();
    let (same, document) = expression_document(&original, block, &allocator)
        .unwrap()
        .into_parts();
    for width in [0, 7, 200] {
        for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
            assert_eq!(print(&document, &options(width, line_ending)), selected);
        }
    }
    assert!(core::ptr::eq(same, &original));
    assert_eq!(core::ptr::from_ref(same.expression().unwrap()), ast);
    let Expression::ParenthesizedExpression(same_parentheses) = same.expression().unwrap() else {
        panic!("same original parentheses")
    };
    assert_eq!(core::ptr::from_ref(&same_parentheses.expression), child);
    assert_eq!(same.source().authored_root().as_ptr(), root.as_ptr());
    assert_eq!(same.source().text().as_ptr(), decoded);
    assert_eq!(
        same.source().decode_map().unwrap().segments().as_ptr(),
        segments
    );
    assert_eq!(same.comments().count(), 0);
    assert_eq!(same.diagnostics().count(), 0);
}

#[derive(Debug, PartialEq, Eq)]
enum Syntax {
    Identifier(std::string::String, std::string::String),
    Numeric(u64, std::string::String),
    String(std::string::String, std::string::String),
    Boolean(bool, std::string::String),
    Parentheses(std::boxed::Box<Syntax>),
    Unary(&'static str, std::boxed::Box<Syntax>),
    Binary(
        &'static str,
        std::boxed::Box<Syntax>,
        std::boxed::Box<Syntax>,
    ),
    Logical(
        &'static str,
        std::boxed::Box<Syntax>,
        std::boxed::Box<Syntax>,
    ),
}

fn fingerprint(original: &RetainedExpression<'_>, expression: &Expression<'_>) -> Syntax {
    let span = original.authored_span(expression.span()).unwrap();
    let spelling = original
        .source()
        .authored_root()
        .get(span.start as usize..span.end as usize)
        .unwrap()
        .to_owned();
    match expression {
        Expression::Identifier(identifier) => {
            Syntax::Identifier(identifier.name.as_str().to_owned(), spelling)
        }
        Expression::NumericLiteral(number) => Syntax::Numeric(number.value.to_bits(), spelling),
        Expression::StringLiteral(string) => {
            Syntax::String(string.value.as_str().to_owned(), spelling)
        }
        Expression::BooleanLiteral(boolean) => Syntax::Boolean(boolean.value, spelling),
        Expression::ParenthesizedExpression(parentheses) => Syntax::Parentheses(
            std::boxed::Box::new(fingerprint(original, &parentheses.expression)),
        ),
        Expression::UnaryExpression(unary) => Syntax::Unary(
            unary.operator.as_str(),
            std::boxed::Box::new(fingerprint(original, &unary.argument)),
        ),
        Expression::BinaryExpression(binary) => Syntax::Binary(
            binary.operator.as_str(),
            std::boxed::Box::new(fingerprint(original, &binary.left)),
            std::boxed::Box::new(fingerprint(original, &binary.right)),
        ),
        Expression::LogicalExpression(logical) => Syntax::Logical(
            logical.operator.as_str(),
            std::boxed::Box::new(fingerprint(original, &logical.left)),
            std::boxed::Box::new(fingerprint(original, &logical.right)),
        ),
        _ => panic!("bounded original fixture AST"),
    }
}

#[test]
fn actual_js_ts_reparse_preserves_semantics_and_fixed_points_for_both_line_endings() {
    for source in [
        "(&#39;//x&#39;\n)",
        "( \n&#39;//x&#39;\r\n )",
        "!\n(&#39;//x&#39;\n)",
        "(&fjlig;\n)",
        "(\n&#39;x&#39;)",
        "&#39;//x&#39; +\r\n1",
        "&#39;//x&#39;\n+1",
        "a\r\n+ tr&#117;e",
        "a\n&& tr&#117;e",
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let allocator = Allocator::default();
            let original = retained(
                &allocator,
                source,
                Span::new(0, source.len() as u32),
                lang,
                true,
            );
            assert_eq!(original.hole(), None);
            assert_eq!(original.comments().count(), 0);
            assert_eq!(original.diagnostics().count(), 0);
            let expected = fingerprint(&original, original.expression().unwrap());
            for width in [0, 7, 200] {
                for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let options = options(width, line_ending);
                    let output = format(source, lang, true, options);
                    let replay_allocator = Allocator::default();
                    let replay = retained(
                        &replay_allocator,
                        &output,
                        Span::new(0, output.len() as u32),
                        lang,
                        true,
                    );
                    assert_eq!(replay.hole(), None);
                    assert_eq!(replay.source_type(), original.source_type());
                    assert_eq!(replay.comments().count(), 0);
                    assert_eq!(replay.diagnostics().count(), 0);
                    assert_eq!(
                        fingerprint(&replay, replay.expression().unwrap()),
                        expected,
                        "{source}: {output}"
                    );
                    assert_eq!(format(&output, lang, true, options), output, "{source}");
                }
            }
        }
    }
}
