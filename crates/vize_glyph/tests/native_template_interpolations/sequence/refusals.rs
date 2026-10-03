//! Whole sequence refusals keep original source, observations and fixed bounds.

use super::{SCRIPTS, assert_preserved, format, options};
use crate::{operands, selected};
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeTemplateRefusal, native_template_document,
};
use vize_l0::Allocator;
use vize_l1::embed::syntax::EmbedHole;

#[test]
fn original_unseparated_object_closer_keeps_earliest_selected_safety_refusal() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!(
            "<template>{{{{a,b}}}}<p>{{{{/*原*/ a,{{b:1}}}}}}</p></template>{script}"
        );
        assert_eq!(
            source.strip_suffix(script).unwrap(),
            "<template>{{a,b}}<p>{{/*原*/ a,{b:1}}}</p></template>"
        );
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let operand = original.get(1).unwrap();
        let syntax = operand.syntax();
        assert_eq!(syntax.hole(), Some(EmbedHole::SafetyAdmission), "{source}");
        assert!(syntax.admitted_expression().is_none());
        assert!(syntax.expression().is_none());
        let diagnostics = syntax.diagnostics().count();
        let comments = syntax.comments().count();
        let view = syntax.source();
        let refs = original.iter().collect::<std::vec::Vec<_>>();
        assert!(matches!(
            native_template_document(&owner, &refs, &arena),
            Err(NativeTemplateRefusal::OperandRejected {
                index: 1,
                hole: Some(EmbedHole::SafetyAdmission),
                ..
            })
        ));
        assert_eq!(syntax.hole(), Some(EmbedHole::SafetyAdmission));
        assert_eq!(syntax.diagnostics().count(), diagnostics);
        assert_eq!(syntax.comments().count(), comments);
        assert!(core::ptr::eq(syntax.source().text(), view.text()));
        assert!(core::ptr::eq(
            syntax.source().authored_root(),
            source.as_str()
        ));
        assert_eq!(operand.content_span().slice(&source), operand.raw_content());
    }
}

#[test]
fn unsupported_sequence_children_or_ancestors_refuse_the_complete_selected_document() {
    for (content, ts_only) in [
        ("this,b", false),
        ("a,this", false),
        ("a,[...b]", false),
        ("a,{b:1} ", false),
        ("a,(b=c)", false),
        ("a,b++", false),
        ("a,await b", false),
        ("a,new B", false),
        ("a,import('b')", false),
        ("a,b=>c", false),
        ("a,b?.c", false),
        ("a,f(...b)", false),
        ("a,1n", false),
        ("a,/x/", false),
        ("a,`x`", false),
        ("a,super.b", false),
        ("f((a,++b))", false),
        ("[(a,++b)]", false),
        ("obj[a,++b]", false),
        ("(0,obj.#m)()", false),
        ("(a,b)?.()", false),
        ("a,&#123;b:1&#125;", false),
        ("a,(b as T)", true),
        ("(a as T),b", true),
        ("a,f<T>(b)", true),
        ("f<T>((a,b))", true),
        ("a,b!", true),
    ] {
        for script in SCRIPTS {
            if ts_only && script.is_empty() {
                continue;
            }
            let arena = Allocator::default();
            let source = vize_l0::cstr!(
                "<template>{{{{a,b}}}}<p>{{{{/*原*/ {content}}}}}</p></template>{script}"
            );
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let syntax = original.get(1).unwrap().syntax();
            assert_eq!(syntax.hole(), None, "{source}");
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
            let view = syntax.source();
            let map = view.decode_map().map(|map| map.segments().as_ptr());
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
            assert_eq!(syntax.source().span(), view.span());
            assert!(core::ptr::eq(syntax.source().text(), view.text()));
            assert!(core::ptr::eq(
                syntax.source().authored_root(),
                source.as_str()
            ));
            assert_eq!(syntax.diagnostics().count(), diagnostics);
            for operand in &original {
                assert_eq!(operand.content_span().slice(&source), operand.raw_content());
            }
        }
    }
}

#[test]
fn sequence_holes_and_trailing_commas_remain_syntax_refusals_after_a_supported_original() {
    for content in ["a,", "a,,b", "(a,)", "(,a)", "a,/*only*/", "f((a,))", ","] {
        for script in SCRIPTS {
            let arena = Allocator::default();
            let source = vize_l0::cstr!(
                "<template>{{{{a,b}}}}<p>{{{{/*keep*/ {content}}}}}</p></template>{script}"
            );
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let operand = original.get(1).unwrap();
            assert!(operand.syntax().hole().is_some(), "{source}");
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
fn non_ascii_sequence_operator_gaps_have_no_plain_whitespace_formatting_authority() {
    for content in [
        "a\u{a0},b",
        "a,\u{a0}b",
        "(a,\u{a0}b)",
        "obj[a,\u{a0}b]",
        "a&#160;,b",
        "a,&#160;b",
        "a,\u{a0}(b,c)",
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
fn genuine_sequence_operands_cannot_be_foreign_missing_extra_reversed_or_duplicated() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!("<template>{{{{a,b}}}}<p>{{{{c,d}}}}</p></template>{script}");
        let owner = selected(&arena, &source);
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
        for foreign in [selected(&arena, &source), selected(&arena, &copied)] {
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
        assert_eq!(a.raw_content(), "a,b");
        assert_eq!(b.raw_content(), "c,d");
    }
}

#[test]
fn genuine_sequence_unary_children_keep_independent_depth_sixteen_and_refuse_seventeen() {
    let supported = vize_l0::cstr!("a,{}b", "!".repeat(15));
    for script in SCRIPTS {
        let source = vize_l0::cstr!("<template>{{{{{supported}}}}}</template>{script}");
        assert_eq!(
            format(&source, options(200, LineEnding::Lf)),
            vize_l0::cstr!("{{{{ a, {}b }}}}", "! ".repeat(15))
        );
        let content = vize_l0::cstr!("a,{}b", "!".repeat(16));
        let source = vize_l0::cstr!("<template>{{{{{content}}}}}</template>{script}");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let original = operands(&owner);
        let operand = original.first().unwrap();
        let syntax = operand.syntax();
        assert_eq!(
            syntax.hole(),
            None,
            "whole parser input stays below the unchanged 31-unit bound"
        );
        assert!(syntax.admitted_expression().is_some());
        let root = core::ptr::from_ref(syntax.expression().unwrap());
        let refs = original.iter().collect::<std::vec::Vec<_>>();
        let Err(NativeTemplateRefusal::Expression {
            refusal: ExpressionRefusal::DepthLimit { span },
            ..
        }) = native_template_document(&owner, &refs, &arena)
        else {
            panic!("actual child at depth seventeen must refuse")
        };
        assert_eq!(span.slice(syntax.source().text()), "b");
        assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), root);
        assert_eq!(operand.content_span().slice(&source), operand.raw_content());
    }
    assert_preserved(&supported);
}
