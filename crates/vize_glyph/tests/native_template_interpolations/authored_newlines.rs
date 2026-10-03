//! Physical source gaps retain the original container's pre-decode boundary.

use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{LineEnding, PrintOptions, native_template_document, print};
use vize_l0::Allocator;

use super::{format, operands, preservation::fingerprint, selected};

#[test]
fn actual_descriptor_requires_physical_lf_after_encoded_quote_literal_slashes() {
    for content in [
        "(&#39;//x&#39;\n)",
        "(&quot;//x&quot;\r\n)",
        "&#39;//x&#39;\n+a",
        "obj[&#39;//x&#39;\r\n]",
    ] {
        for script in ["", "<script setup lang=ts>let a=1</script>"] {
            let source = vize_l0::cstr!(
                "<!--前--><template>{}{}{}</template>{script}",
                "{{",
                content,
                "}}"
            );
            let arena = Allocator::default();
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let syntax = original.first().unwrap().syntax();
            assert_eq!(syntax.hole(), None, "{source}");
            assert!(syntax.admitted_expression().is_some());
            assert!(syntax.source().decode_map().is_some());
            // The retained parser sees string content, not an original comment.
            assert_eq!(syntax.comments().count(), 0);
            match syntax.expression().unwrap() {
                Expression::ParenthesizedExpression(parentheses) => {
                    assert!(
                        matches!(&parentheses.expression, Expression::StringLiteral(value) if value.value.as_str() == "//x")
                    );
                }
                Expression::BinaryExpression(binary) => {
                    assert!(
                        matches!(&binary.left, Expression::StringLiteral(value) if value.value.as_str() == "//x")
                    );
                }
                Expression::ComputedMemberExpression(member) => {
                    assert!(
                        matches!(&member.expression, Expression::StringLiteral(value) if value.value.as_str() == "//x")
                    );
                }
                _ => panic!("incorrect original fixture shape"),
            }
            // These complete authored controls exercise the actual Descriptor.
            // CR alone does not terminate its original raw // boundary.
            for terminator in ["", "\r"] {
                let mutated = content
                    .replace("\r\n", terminator)
                    .replace('\n', terminator);
                let mutated = vize_l0::cstr!(
                    "<!--前--><template>{}{}{}</template>{script}",
                    "{{",
                    mutated,
                    "}}"
                );
                let descriptor = super::Vue.observe_descriptor(
                    &arena,
                    &mutated,
                    super::DescriptorOptions {
                        version: super::VueVersion::V3,
                        dialect: super::VueDialect::Vue,
                        template: super::SurfaceParseOptions::default(),
                    },
                );
                assert!(descriptor.admitted().is_err(), "{mutated}");
                assert!(descriptor.issues().iter().any(|issue| issue.code
                    == vize_l1::container::vue::DescriptorIssueCode::UnsupportedBoundary));
                assert!(core::ptr::eq(descriptor.source(), mutated.as_str()));
            }
        }
    }
}

#[test]
fn selected_internal_authored_lf_and_crlf_keep_complete_output_at_every_width() {
    for (content, expected) in [
        ("(&#39;//x&#39;\n)", "(&#39;//x&#39;\n)"),
        ("(&quot;//x&quot;\r\n)", "(&quot;//x&quot;\r\n)"),
        ("&#39;//x&#39;\n+a", "&#39;//x&#39;\n+\n    a"),
        ("&quot;//x&quot;\r\n+a", "&quot;//x&quot;\r\n+\n    a"),
    ] {
        for script in ["", "<script setup lang=ts>let a=1</script>"] {
            let source = vize_l0::cstr!(
                "<!--前--><template>{}{}{}</template>{script}",
                "{{",
                content,
                "}}"
            );
            for width in [0, 1, 7, 80, 200] {
                assert_eq!(
                    format(
                        &source,
                        PrintOptions {
                            width,
                            ..PrintOptions::default()
                        }
                    ),
                    vize_l0::cstr!("{}\n  {expected}\n{}", "{{", "}}"),
                    "{source} / width {width}"
                );
            }
        }
    }
    assert_eq!(
        format(
            "<template>{{&#39;//x&#39;\n+a}}</template>",
            PrintOptions {
                line_ending: LineEnding::CrLf,
                ..PrintOptions::default()
            }
        ),
        "{{\r\n  &#39;//x&#39;\n+\r\n    a\r\n}}"
    );
    // An identity source has no decoding map and keeps ordinary gap layout.
    assert_eq!(
        format("<template>{{a\n+b}}</template>", PrintOptions::default()),
        "{{ a + b }}"
    );
}

#[test]
fn internal_authored_newline_outputs_reselect_reparse_and_reach_fixed_points() {
    for content in [
        "(&#39;//x&#39;\n)",
        "(&quot;//x&quot;\r\n)",
        "&#39;//x&#39;\n+a",
        "&quot;//x&quot;\r\n+a",
        "a+(&#39;//x&#39;\n)",
        "(&#39;//x&#39;\n/*kept\r\n*/)",
        "a &#47;*kept\r\n*&#47; +b",
    ] {
        for script in ["", "<script setup lang=ts>let a=1</script>"] {
            let source = vize_l0::cstr!(
                "<!--前--><template>{}{}{}</template>{script}",
                "{{",
                content,
                "}}"
            );
            let arena = Allocator::default();
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let syntax = original.first().unwrap().syntax();
            let expected = fingerprint(syntax, syntax.expression().unwrap());
            let comments = syntax
                .comments()
                .map(|comment| {
                    let span = comment.authored_span().unwrap();
                    (
                        comment.kind(),
                        comment.text().unwrap().to_owned(),
                        span.slice(&source).to_owned(),
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
                    let replay_owner = selected(&replay_arena, &replay_source);
                    let replay_operands = operands(&replay_owner);
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
                                    span.slice(&replay_source).to_owned(),
                                )
                            })
                            .collect::<std::vec::Vec<_>>(),
                        comments,
                        "{replay_source}"
                    );
                    assert_eq!(format(&replay_source, options), output, "{replay_source}");
                }
            }
        }
    }
}

#[test]
fn physical_gap_retention_borrows_the_original_owner_ast_comments_map_and_source() {
    let source = "<!--前--><template>{{(&#39;//x&#39;\n/*kept\r\n*/)}}<p>{{&quot;//y&quot;\r\n+a}}</p></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let original = operands(&owner);
    let states = original
        .iter()
        .map(|operand| {
            let syntax = operand.syntax();
            (
                core::ptr::from_ref(syntax.expression().unwrap()),
                syntax
                    .comments()
                    .map(|comment| {
                        (
                            comment.decoded_span().unwrap(),
                            comment.authored_span().unwrap(),
                            comment.text().unwrap().as_ptr(),
                        )
                    })
                    .collect::<std::vec::Vec<_>>(),
                syntax.source().decode_map().unwrap().segments(),
                syntax.source().text().as_ptr(),
                operand.raw_content().as_ptr(),
            )
        })
        .collect::<std::vec::Vec<_>>();
    let refs = original.iter().collect::<std::vec::Vec<_>>();
    let document = native_template_document(&owner, &refs, &arena).unwrap();
    assert_eq!(
        print(document.document(), &PrintOptions::default()),
        "{{\n  (&#39;//x&#39;\n/*kept\r\n*/)\n}}<p>{{\n    &quot;//y&quot;\r\n+\n      a\n  }}</p>"
    );
    for width in [0, 1, 7, 80, 200] {
        for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
            print(
                document.document(),
                &PrintOptions {
                    width,
                    line_ending,
                    ..PrintOptions::default()
                },
            );
            assert!(core::ptr::eq(document.original(), &owner));
            assert!(core::ptr::eq(document.operands(), refs.as_slice()));
            for (operand, state) in original.iter().zip(&states) {
                let syntax = operand.syntax();
                assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), state.0);
                assert_eq!(
                    syntax
                        .comments()
                        .map(|comment| (
                            comment.decoded_span().unwrap(),
                            comment.authored_span().unwrap(),
                            comment.text().unwrap().as_ptr()
                        ))
                        .collect::<std::vec::Vec<_>>(),
                    state.1
                );
                assert!(core::ptr::eq(
                    syntax.source().decode_map().unwrap().segments(),
                    state.2
                ));
                assert_eq!(syntax.source().text().as_ptr(), state.3);
                assert_eq!(operand.raw_content().as_ptr(), state.4);
                assert_eq!(operand.content_span().slice(source), operand.raw_content());
            }
        }
    }
}
