//! Actual selected-source reparse, with independent typed syntax fingerprints.

use oxc_ast::ast::{
    ArrayExpressionElement, Expression, ObjectPropertyKind, PropertyKey, PropertyKind,
};
use oxc_span::GetSpan;
use vize_glyph::native_doc::{LineEnding, PrintOptions, native_template_document, print};
use vize_l0::Allocator;
use vize_l1::embed::syntax::RetainedExpression;

use super::{format, operands, selected};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Syntax {
    Atom(&'static str, std::string::String, std::string::String),
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
    Infix(
        &'static str,
        std::boxed::Box<Syntax>,
        std::boxed::Box<Syntax>,
    ),
}

pub(super) fn fingerprint(
    original: &RetainedExpression<'_>,
    expression: &Expression<'_>,
) -> Syntax {
    let span = original.authored_span(expression.span()).unwrap();
    let spelling = original
        .source()
        .authored_root()
        .get(span.start as usize..span.end as usize)
        .unwrap()
        .to_owned();
    match expression {
        Expression::Identifier(identifier) => {
            Syntax::Atom("identifier", identifier.name.as_str().to_owned(), spelling)
        }
        Expression::NumericLiteral(number) => {
            Syntax::Atom("number", number.value.to_bits().to_string(), spelling)
        }
        Expression::StringLiteral(string) => {
            Syntax::Atom("string", string.value.as_str().to_owned(), spelling)
        }
        Expression::BooleanLiteral(boolean) => {
            Syntax::Atom("boolean", boolean.value.to_string(), spelling)
        }
        Expression::NullLiteral(_) => Syntax::Atom("null", "null".to_owned(), spelling),
        Expression::ParenthesizedExpression(parentheses) => Syntax::Parentheses(
            std::boxed::Box::new(fingerprint(original, &parentheses.expression)),
        ),
        Expression::UnaryExpression(unary) => Syntax::Unary(
            unary.operator.as_str(),
            std::boxed::Box::new(fingerprint(original, &unary.argument)),
        ),
        Expression::StaticMemberExpression(member) => {
            let property = original.authored_span(member.property.span()).unwrap();
            Syntax::StaticMember(
                member.optional,
                std::boxed::Box::new(fingerprint(original, &member.object)),
                member.property.name.as_str().to_owned(),
                original
                    .source()
                    .authored_root()
                    .get(property.start as usize..property.end as usize)
                    .unwrap()
                    .to_owned(),
            )
        }
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
                    ArrayExpressionElement::Elision(elision) => Syntax::Elision({
                        let span = original.authored_span(elision.span()).unwrap();
                        original
                            .source()
                            .authored_root()
                            .get(span.start as usize..span.end as usize)
                            .unwrap()
                            .to_owned()
                    }),
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
        Expression::BinaryExpression(binary) => Syntax::Infix(
            binary.operator.as_str(),
            std::boxed::Box::new(fingerprint(original, &binary.left)),
            std::boxed::Box::new(fingerprint(original, &binary.right)),
        ),
        Expression::LogicalExpression(logical) => Syntax::Infix(
            logical.operator.as_str(),
            std::boxed::Box::new(fingerprint(original, &logical.left)),
            std::boxed::Box::new(fingerprint(original, &logical.right)),
        ),
        _ => panic!("unsupported fixture family"),
    }
}

#[test]
fn actual_selected_js_ts_reparse_keeps_nodes_literals_operators_parentheses_and_comments() {
    for content in [
        "((a+0xCA_FE))*1_000",
        "a /*x*/ + ('x\\x20y'&amp;&amp;true) //tail\n",
        "//before\n a +b",
        "a + //inside\n b",
        "a //tail&#10;\n",
        "a &#47;*//x*&#47;\n",
        "&#39;//x&#39;\n",
        "a &#47;*encoded*&#47; + '&amp;amp;'",
        "&#32;&fjlig; &amp;&amp; (tr&#117;e)&#32;",
        "a&#32;+&#9;b",
        "!ready",
        "~bits",
        "+1",
        "-1",
        "typeof value",
        "void 0",
        "delete 0",
        "+ +a",
        "- -a",
        "a + +b",
        "a - -b",
        "-(a+b)",
        "(-a)**b",
        "-(a**b)",
        "!(a&amp;&amp;b)",
        "&#33;ready",
        "&#43;&#32;&#43;a",
        "!&#9;ready",
        "! /*kept\r\n*/ ready",
    ] {
        assert_preserved(content);
    }
}

pub(super) fn assert_preserved(content: &str) {
    for script in ["", "<script setup lang=ts>let a=1</script>"] {
        let source = vize_l0::cstr!("<template><p>{{{{{content}}}}}</p></template>{script}");
        let arena = Allocator::default();
        let original = selected(&arena, &source);
        let original_operands = operands(&original);
        let syntax = original_operands.first().unwrap().syntax();
        let expected = fingerprint(syntax, syntax.expression().unwrap());
        let comments = syntax
            .comments()
            .map(|comment| {
                let span = comment.authored_span().unwrap();
                (
                    comment.kind(),
                    comment.text().unwrap().to_owned(),
                    source
                        .get(span.start as usize..span.end as usize)
                        .unwrap()
                        .to_owned(),
                )
            })
            .collect::<std::vec::Vec<_>>();
        for width in [0, 1, 7, 80, 200] {
            for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                let options = PrintOptions {
                    width,
                    line_ending,
                    ..PrintOptions::default()
                };
                let output = format(&source, options);
                let replay_source = vize_l0::cstr!("<template>{output}</template>{script}");
                let replay_arena = Allocator::default();
                let replay = selected(&replay_arena, &replay_source);
                let replay_operands = operands(&replay);
                let replay = replay_operands.first().unwrap().syntax();
                assert_eq!(replay.hole(), None, "{replay_source}");
                assert_eq!(replay.source_type(), syntax.source_type());
                assert_eq!(
                    fingerprint(replay, replay.expression().unwrap()),
                    expected,
                    "{replay_source}"
                );
                assert_eq!(
                    replay
                        .comments()
                        .map(|comment| {
                            let span = comment.authored_span().unwrap();
                            (
                                comment.kind(),
                                comment.text().unwrap().to_owned(),
                                replay_source
                                    .get(span.start as usize..span.end as usize)
                                    .unwrap()
                                    .to_owned(),
                            )
                        })
                        .collect::<std::vec::Vec<_>>(),
                    comments,
                    "{replay_source}"
                );
                assert_eq!(format(&replay_source, options), output);
            }
        }
    }
}

#[test]
fn original_owner_ast_comment_map_and_content_addresses_remain_unchanged() {
    let arena = Allocator::default();
    let source = "<!--前--><template><p>{{ /*原*/ 日本 &amp;&amp; ok }}</p></template>";
    let original = selected(&arena, source);
    let original_operands = operands(&original);
    let operand = original_operands.first().unwrap();
    let syntax = operand.syntax();
    let ast = core::ptr::from_ref(syntax.expression().unwrap());
    let comment = syntax.comments().next().unwrap();
    let comment_text = comment.text().unwrap().as_ptr();
    let map = syntax.source().decode_map().unwrap();
    let content = operand.raw_content().as_ptr();
    let refs = original_operands.iter().collect::<std::vec::Vec<_>>();
    let document = native_template_document(&original, &refs, &arena).unwrap();
    assert_eq!(
        print(document.document(), &PrintOptions::default()),
        "<p>{{ /*原*/ 日本 &amp;&amp; ok }}</p>"
    );
    assert!(core::ptr::eq(
        *document.operands().first().unwrap(),
        operand
    ));
    assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), ast);
    assert_eq!(
        syntax.comments().next().unwrap().text().unwrap().as_ptr(),
        comment_text
    );
    assert!(core::ptr::eq(
        syntax.source().decode_map().unwrap().segments(),
        map.segments()
    ));
    assert_eq!(operand.raw_content().as_ptr(), content);
    assert_eq!(operand.content_span().slice(source), operand.raw_content());
    assert_eq!(
        operand.full_span().slice(source),
        "{{ /*原*/ 日本 &amp;&amp; ok }}"
    );
}
