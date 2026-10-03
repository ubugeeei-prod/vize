//! Actual selected-source reparse, with independent typed syntax fingerprints.

use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_glyph::native_doc::{LineEnding, PrintOptions, native_template_document, print};
use vize_l0::Allocator;
use vize_l1::embed::syntax::RetainedExpression;

use super::{format, operands, selected};

#[derive(Debug, PartialEq, Eq)]
enum Syntax {
    Atom(&'static str, std::string::String, std::string::String),
    Parentheses(std::boxed::Box<Syntax>),
    Infix(
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
    ] {
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
            for width in [0, 1, 7, 80] {
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
