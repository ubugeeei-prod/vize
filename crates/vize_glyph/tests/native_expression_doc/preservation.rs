//! Independent typed AST/comment fingerprints across a real second parse.

use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_glyph::native_doc::PrintOptions;
use vize_l0::{Allocator, Span};
use vize_l1::embed::{Lang, syntax::RetainedExpression};

use super::{format, retained};

#[derive(Debug, PartialEq, Eq)]
enum Syntax {
    Identifier(std::string::String, std::string::String),
    Numeric(u64, std::string::String),
    String(std::string::String, std::string::String),
    Boolean(bool, std::string::String),
    Null(std::string::String),
    Parentheses(std::boxed::Box<Syntax>),
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
        Expression::NullLiteral(_) => Syntax::Null(spelling),
        Expression::ParenthesizedExpression(parentheses) => Syntax::Parentheses(
            std::boxed::Box::new(fingerprint(original, &parentheses.expression)),
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
        _ => panic!("independent fixture family"),
    }
}

#[test]
fn real_js_ts_reparse_preserves_node_operator_parenthesis_literal_and_comment_semantics() {
    for (source, decode) in [
        (" (( a+0xCA_FE )) * 1_000 ", false),
        ("a /*x*/ + ('x\\x20y'&&true) //t", false),
        ("a //x\n + (b??null)", false),
        ("a /*x\r\ny*/ + b", false),
        ("&fjlig; &amp;&amp; ( tr&#117;e )", true),
        ("a &#47;*x*&#47; + 'a&amp;b'", true),
        ("a&#32;+&#9;b", true),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let allocator = Allocator::default();
            let original = retained(
                &allocator,
                source,
                Span::new(0, source.len() as u32),
                lang,
                decode,
            );
            let expected = fingerprint(&original, original.expression().unwrap());
            let comments = original
                .comments()
                .map(|comment| (comment.kind(), comment.text().unwrap().to_owned()))
                .collect::<std::vec::Vec<_>>();
            for width in [0, 7, 20, 200] {
                let output = format(
                    source,
                    lang,
                    decode,
                    PrintOptions {
                        width,
                        ..PrintOptions::default()
                    },
                );
                let replay_allocator = Allocator::default();
                let replay = retained(
                    &replay_allocator,
                    &output,
                    Span::new(0, output.len() as u32),
                    lang,
                    decode,
                );
                assert_eq!(replay.hole(), None);
                assert_eq!(replay.diagnostics().count(), 0);
                assert_eq!(replay.source_type(), original.source_type());
                assert_eq!(
                    fingerprint(&replay, replay.expression().unwrap()),
                    expected,
                    "{source}: {output}"
                );
                assert_eq!(
                    replay
                        .comments()
                        .map(|comment| (comment.kind(), comment.text().unwrap().to_owned()))
                        .collect::<std::vec::Vec<_>>(),
                    comments,
                    "{source}: {output}"
                );
            }
        }
    }
}
