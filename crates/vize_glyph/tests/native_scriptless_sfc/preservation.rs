//! Real whole Descriptor replay and independent typed AST/authored-comment preservation.

use super::{assert_fixed, fingerprint, options};
use oxc_ast::ast::CommentKind;
use vize_glyph::native_doc::{LineEnding, observe_native_sfc_in};
use vize_l0::Allocator;
use vize_l1::embed::syntax::RetainedExpression;

pub(super) fn comments(
    original: &RetainedExpression<'_>,
) -> std::vec::Vec<(CommentKind, std::string::String, std::string::String)> {
    original
        .comments()
        .map(|comment| {
            let span = comment.authored_span().unwrap();
            (
                comment.kind(),
                comment.text().unwrap().to_owned(),
                span.slice(original.source().authored_root()).to_owned(),
            )
        })
        .collect()
}

#[test]
fn complete_descriptor_replay_preserves_typed_syntax_literals_operators_and_authored_comments() {
    for content in [
        "((a+0xCA_FE))*1_000",
        "!(a&&null)",
        "obj.x",
        "obj[a]",
        "/*#__PURE__*/f&#40;0x2_A&#110;&#41;",
        "[a,,b,]",
        "{a:1n,b:'x'} ",
        "a?b:c",
        "a,/*x*/b",
        "&#123;a:1n&#125;",
        "a &#47;*encoded*&#47;+ '&amp;amp;'",
        "(&#39;//x&#39;\n/*kept\r\n*/)",
        "obj[&quot;//x&quot;\r\n]",
    ] {
        let source = format!(
            "\u{feff}<!--前-->\r\n<template lang='html'>{{{{{content}}}}}<p>{{{{1n}}}}</p></template><!--尾-->\n"
        );
        for width in [0, 7, 200] {
            for indent in [0, 2, 4] {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_fixed(&source, options(width, indent, ending));
                }
            }
        }
    }
}

#[test]
fn authored_comment_spelling_change_is_detected_even_when_decoded_syntax_is_equal() {
    let source = "<template>{{a &#47;*x*&#47;+b}}</template>";
    let arena = Allocator::default();
    let options = options(200, 2, LineEnding::Lf);
    let original = observe_native_sfc_in(&arena, source, options);
    let output = original.format().unwrap().code;
    assert_eq!(output, "<template>{{ a &#47;*x*&#47;+ b }}</template>");
    let changed = output.replace("&#47;", "/");
    let second_arena = Allocator::default();
    let edited = observe_native_sfc_in(&second_arena, &changed, options);
    assert!(edited.refusal().is_none());
    let first = original.operands()[0].syntax();
    let second = edited.operands()[0].syntax();
    assert_eq!(
        fingerprint::fingerprint(first, first.expression().unwrap()),
        fingerprint::fingerprint(second, second.expression().unwrap())
    );
    assert_eq!(
        first.comments().next().unwrap().text(),
        second.comments().next().unwrap().text()
    );
    assert_ne!(comments(first), comments(second));
    assert_fixed(source, options);
}
