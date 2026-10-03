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
    Unary(&'static str, std::boxed::Box<Syntax>),
    StaticMember(
        bool,
        std::boxed::Box<Syntax>,
        std::string::String,
        std::string::String,
    ),
    ComputedMember(bool, std::boxed::Box<Syntax>, std::boxed::Box<Syntax>),
    Call(
        bool,
        bool,
        bool,
        std::boxed::Box<Syntax>,
        std::vec::Vec<Syntax>,
    ),
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
    let spelling = authored(original, expression.span());
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
        Expression::UnaryExpression(unary) => Syntax::Unary(
            unary.operator.as_str(),
            std::boxed::Box::new(fingerprint(original, &unary.argument)),
        ),
        Expression::StaticMemberExpression(member) => Syntax::StaticMember(
            member.optional,
            std::boxed::Box::new(fingerprint(original, &member.object)),
            member.property.name.as_str().to_owned(),
            authored(original, member.property.span()),
        ),
        Expression::ComputedMemberExpression(member) => Syntax::ComputedMember(
            member.optional,
            std::boxed::Box::new(fingerprint(original, &member.object)),
            std::boxed::Box::new(fingerprint(original, &member.expression)),
        ),
        Expression::CallExpression(call) => Syntax::Call(
            call.optional,
            call.pure,
            call.type_arguments.is_some(),
            std::boxed::Box::new(fingerprint(original, &call.callee)),
            call.arguments
                .iter()
                .map(|argument| fingerprint(original, argument.as_expression().unwrap()))
                .collect(),
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

fn authored(original: &RetainedExpression<'_>, span: oxc_span::Span) -> std::string::String {
    let span = original.authored_span(span).unwrap();
    original
        .source()
        .authored_root()
        .get(span.start as usize..span.end as usize)
        .unwrap()
        .to_owned()
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
        assert_preserved(source, decode);
    }
}

pub(super) fn assert_preserved(source: &str, decode: bool) {
    for lang in [Lang::Js, Lang::Ts] {
        let allocator = Allocator::default();
        let original = retained(
            &allocator,
            source,
            Span::new(0, source.len() as u32),
            lang,
            decode,
        );
        assert_eq!(original.hole(), None, "{source}");
        assert_eq!(original.diagnostics().count(), 0, "{source}");
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
