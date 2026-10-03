//! Whole selected conditional refusals keep original observations normally owned.

use super::{SCRIPTS, assert_preserved, format, options};
use crate::{operands, selected};
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeTemplateRefusal, native_template_document,
};
use vize_l0::Allocator;

#[test]
fn unsupported_conditional_test_consequent_or_alternate_refuses_the_complete_document() {
    for (content, ts_only) in [
        ("a?.x?b:c", false),
        ("a?b?.x:c", false),
        ("a?b:c?.x", false),
        ("f(...a)?b:c", false),
        ("a?f(...b):c", false),
        ("a?b:f(...c)", false),
        ("(a=b)?c:d", false),
        ("a?(b=c):d", false),
        ("a?b:(c=d)", false),
        ("[...a]?b:c", false),
        ("a?{b:1}:c", false),
        ("a?b:()=>c", false),
        ("this?b:c", false),
        ("a?new B:c", false),
        ("a?b:import('c')", false),
        ("a?b:super.c", false),
        ("a?b:c++", false),
        ("a?b:&#91;...c&#93;", false),
        ("(a as T)?b:c", true),
        ("a?(b as T):c", true),
        ("a?b:(c as T)", true),
        ("f<T>(a)?b:c", true),
        ("a?f<T>(b):c", true),
        ("a?b:f<T>(c)", true),
    ] {
        for script in SCRIPTS {
            if ts_only && script.is_empty() {
                continue;
            }
            let arena = Allocator::default();
            let source = vize_l0::cstr!(
                "<template>{{{{a?b:c}}}}<p>{{{{/*原*/ {content}}}}}</p></template>{script}"
            );
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let syntax = original.get(1).unwrap().syntax();
            assert!(syntax.admitted_expression().is_some(), "{source}");
            let ast = core::ptr::from_ref(syntax.expression().unwrap());
            let comments = syntax
                .comments()
                .map(|comment| {
                    (
                        comment.kind(),
                        comment.decoded_span().unwrap(),
                        comment.authored_span().unwrap(),
                        comment.text().unwrap().as_ptr(),
                    )
                })
                .collect::<std::vec::Vec<_>>();
            let map = syntax
                .source()
                .decode_map()
                .map(|map| map.segments().as_ptr());
            let diagnostics = syntax.diagnostics().count();
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
            assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), ast);
            assert_eq!(
                syntax
                    .comments()
                    .map(|comment| (
                        comment.kind(),
                        comment.decoded_span().unwrap(),
                        comment.authored_span().unwrap(),
                        comment.text().unwrap().as_ptr(),
                    ))
                    .collect::<std::vec::Vec<_>>(),
                comments
            );
            assert_eq!(
                syntax
                    .source()
                    .decode_map()
                    .map(|map| map.segments().as_ptr()),
                map
            );
            assert_eq!(syntax.diagnostics().count(), diagnostics);
            for operand in &original {
                assert_eq!(operand.content_span().slice(&source), operand.raw_content());
            }
        }
    }
}

#[test]
fn incomplete_conditionals_keep_syntax_holes_and_refuse_after_a_supported_original() {
    for content in ["a?b:", "a?:c", "a?b", "a?b:c?"] {
        for script in SCRIPTS {
            let arena = Allocator::default();
            let source = vize_l0::cstr!(
                "<template>{{{{a?b:c}}}}<p>{{{{/*kept*/ {content}}}}}</p></template>{script}"
            );
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let operand = original.get(1).unwrap();
            let diagnostics = operand.syntax().diagnostics().count();
            let comments = operand.syntax().comments().count();
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
            assert_eq!(operand.syntax().diagnostics().count(), diagnostics);
            assert_eq!(operand.syntax().comments().count(), comments);
            assert_eq!(operand.content_span().slice(&source), operand.raw_content());
        }
    }
}

#[test]
fn original_non_ascii_operator_gaps_do_not_inherit_plain_gap_formatting_authority() {
    for content in [
        "a\u{a0}?b:c",
        "a?\u{a0}b:c",
        "a?b\u{a0}:c",
        "a?b:\u{a0}c",
        "a?&#160;b:c",
    ] {
        for script in SCRIPTS {
            let arena = Allocator::default();
            let source = vize_l0::cstr!("<template>{{{{{content}}}}}</template>{script}");
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let operand = original.first().unwrap();
            let syntax = operand.syntax();
            assert_eq!(syntax.hole(), None);
            let root = core::ptr::from_ref(syntax.expression().unwrap());
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
            assert_eq!(operand.content_span().slice(&source), operand.raw_content());
        }
    }
}

#[test]
fn genuine_conditional_operands_cannot_be_foreign_missing_extra_reversed_or_duplicated() {
    let arena = Allocator::default();
    let source = "<template>{{a?b:c}}<p>{{d?e:f}}</p></template>";
    let owner = selected(&arena, source);
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
    for foreign in [selected(&arena, source), selected(&arena, &copied)] {
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
    assert_eq!(a.raw_content(), "a?b:c");
    assert_eq!(b.raw_content(), "d?e:f");
}

#[test]
fn independent_genuine_conditional_unary_depth_keeps_provider_and_document_limits() {
    let supported = vize_l0::cstr!("{}{}c", "a?b:".repeat(3), "!".repeat(13));
    for script in SCRIPTS {
        let source = vize_l0::cstr!("<template>{{{{{supported}}}}}</template>{script}");
        assert_eq!(
            format(&source, options(200, LineEnding::Lf)),
            vize_l0::cstr!("{{{{ {}{}c }}}}", "a ? b : ".repeat(3), "! ".repeat(13))
        );
        let content = vize_l0::cstr!("{}{}c", "a?b:".repeat(3), "!".repeat(14));
        let source = vize_l0::cstr!("<template>{{{{{content}}}}}</template>{script}");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let operand = original.first().unwrap();
        let syntax = operand.syntax();
        assert_eq!(syntax.hole(), None);
        assert!(syntax.admitted_expression().is_some());
        let root = core::ptr::from_ref(syntax.expression().unwrap());
        let refs = original.iter().collect::<std::vec::Vec<_>>();
        let Err(NativeTemplateRefusal::Expression {
            refusal: ExpressionRefusal::DepthLimit { span },
            ..
        }) = native_template_document(&owner, &refs, &arena)
        else {
            panic!("original admitted depth seventeen must refuse")
        };
        assert_eq!(span.slice(syntax.source().text()), "c");
        assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), root);
        assert_eq!(operand.content_span().slice(&source), operand.raw_content());
    }
    assert_preserved(&supported);
}
