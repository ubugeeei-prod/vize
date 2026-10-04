//! Independent typed expression fingerprints for whole original SFC replay.

use oxc_ast::ast::{
    ArrayExpressionElement, BigintBase, Expression, ObjectPropertyKind, PropertyKey, PropertyKind,
};
use oxc_span::GetSpan;
use vize_l1::embed::syntax::RetainedExpression;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Syntax {
    Identifier(std::string::String, std::string::String),
    Numeric(u64, std::string::String),
    BigInt(
        BigintBase,
        std::string::String,
        Option<std::string::String>,
        std::string::String,
    ),
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
    Object(std::vec::Vec<Syntax>),
    Property(
        PropertyKind,
        bool,
        bool,
        bool,
        std::boxed::Box<Syntax>,
        std::boxed::Box<Syntax>,
    ),
    PropertyKey(&'static str, std::string::String, std::string::String),
    Sequence(std::vec::Vec<Syntax>),
    Array(std::vec::Vec<Syntax>),
    Elision(std::string::String),
    Conditional(
        std::boxed::Box<Syntax>,
        std::boxed::Box<Syntax>,
        std::boxed::Box<Syntax>,
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

pub(super) fn fingerprint(
    original: &RetainedExpression<'_>,
    expression: &Expression<'_>,
) -> Syntax {
    let spelling = authored(original, expression.span());
    match expression {
        Expression::Identifier(identifier) => {
            Syntax::Identifier(identifier.name.as_str().to_owned(), spelling)
        }
        Expression::NumericLiteral(number) => Syntax::Numeric(number.value.to_bits(), spelling),
        Expression::BigIntLiteral(bigint) => Syntax::BigInt(
            bigint.base,
            bigint.value.as_str().to_owned(),
            bigint.raw.as_ref().map(|raw| raw.as_str().to_owned()),
            spelling,
        ),
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
        Expression::ObjectExpression(object) => Syntax::Object(
            object
                .properties
                .iter()
                .map(|property| {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        panic!("independent explicit property fixture")
                    };
                    let key_span = original.authored_span(property.key.span()).unwrap();
                    let authored = original
                        .source()
                        .authored_root()
                        .get(key_span.start as usize..key_span.end as usize)
                        .unwrap()
                        .to_owned();
                    let (kind, value) = match &property.key {
                        PropertyKey::StaticIdentifier(key) => {
                            ("IdentifierName", key.name.as_str().to_owned())
                        }
                        PropertyKey::StringLiteral(key) => {
                            ("StringLiteral", key.value.as_str().to_owned())
                        }
                        PropertyKey::NumericLiteral(key) => {
                            ("NumericLiteral", key.value.to_bits().to_string())
                        }
                        _ => panic!("independent static key fixture"),
                    };
                    Syntax::Property(
                        property.kind,
                        property.method,
                        property.shorthand,
                        property.computed,
                        std::boxed::Box::new(Syntax::PropertyKey(kind, value, authored)),
                        std::boxed::Box::new(fingerprint(original, &property.value)),
                    )
                })
                .collect(),
        ),
        Expression::SequenceExpression(sequence) => Syntax::Sequence(
            sequence
                .expressions
                .iter()
                .map(|child| fingerprint(original, child))
                .collect(),
        ),
        Expression::ArrayExpression(array) => Syntax::Array(
            array
                .elements
                .iter()
                .map(|element| match element {
                    ArrayExpressionElement::Elision(elision) => {
                        Syntax::Elision(authored(original, elision.span()))
                    }
                    _ => fingerprint(original, element.as_expression().unwrap()),
                })
                .collect(),
        ),
        Expression::ConditionalExpression(conditional) => Syntax::Conditional(
            std::boxed::Box::new(fingerprint(original, &conditional.test)),
            std::boxed::Box::new(fingerprint(original, &conditional.consequent)),
            std::boxed::Box::new(fingerprint(original, &conditional.alternate)),
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
