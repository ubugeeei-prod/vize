//! Complete array refusals retain genuine original syntax and source owners.

use super::{SCRIPTS, assert_preserved, format, options};
use crate::{operands, selected};
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeTemplateRefusal, native_template_document,
};
use vize_l0::Allocator;

#[test]
fn spread_or_unsupported_array_descendants_refuse_the_complete_selected_document() {
    for (content, ts_only) in [
        ("[...a]", false),
        ("[...a,]", false),
        ("[,a,...b]", false),
        ("[a,...b,c]", false),
        ("[a,1n]", false),
        ("[a,/x/]", false),
        ("[a,`x`]", false),
        ("[a,()=>b]", false),
        ("[a,function(){}]", false),
        ("[a,{...b}]", false),
        ("[a,(b=c)]", false),
        ("[a,(b,fn(...c))]", false),
        ("[a,this]", false),
        ("[a,new B]", false),
        ("[a,import('b')]", false),
        ("[a,super.b]", false),
        ("[a,b++]", false),
        ("[a,b?.c]", false),
        ("[a,f(...b)]", false),
        ("&#91;a&#44;&#46;&#46;&#46;b&#93;", false),
        ("[a,(b as T)]", true),
        ("[a,f<T>(b)]", true),
        ("[a,b!]", true),
    ] {
        for script in SCRIPTS {
            if ts_only && script.is_empty() {
                continue;
            }
            let arena = Allocator::default();
            let source = vize_l0::cstr!(
                "<template>{{{{[a,,]}}}}<p>{{{{/*原*/ {content}}}}}</p></template>{script}"
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
fn malformed_arrays_keep_their_syntax_holes_after_a_supported_sparse_original() {
    for content in ["[", "[a,b", "[...]", "[a + ,b]", "[a,,b,)"] {
        for script in SCRIPTS {
            let arena = Allocator::default();
            let source = vize_l0::cstr!(
                "<template>{{{{[a,,]}}}}<p>{{{{/*keep*/ {content}}}}}</p></template>{script}"
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
fn non_ascii_array_or_elision_gaps_have_no_plain_whitespace_formatting_authority() {
    for content in [
        "[\u{a0}a]",
        "[a\u{a0},b]",
        "[a,\u{a0}b]",
        "[,\u{a0},a]",
        "[a,,\u{a0}]",
        "[a,&#160;b]",
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
fn sparse_array_operands_cannot_be_foreign_missing_extra_reversed_or_duplicated() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source =
            vize_l0::cstr!("<template>{{{{[,a]}}}}<p>{{{{[b,,]}}}}</p></template>{script}");
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
        assert_eq!(a.raw_content(), "[,a]");
        assert_eq!(b.raw_content(), "[b,,]");
    }
}

#[test]
fn genuine_expression_and_elision_children_share_the_independent_immutable_depth_bound() {
    for (supported, body, refused, leaf) in [
        (
            vize_l0::cstr!("[[[{}c]]]", "!".repeat(13)),
            vize_l0::cstr!("[ [ [ {}c ] ] ]", "! ".repeat(13)),
            vize_l0::cstr!("[[[{}c]]]", "!".repeat(14)),
            "c",
        ),
        (
            vize_l0::cstr!("{}[[[,]]]", "!".repeat(13)),
            vize_l0::cstr!("{}[ [ [ , ] ] ]", "! ".repeat(13)),
            vize_l0::cstr!("{}[[[,]]]", "!".repeat(14)),
            ",",
        ),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!("<template>{{{{{supported}}}}}</template>{script}");
            assert_eq!(
                format(&source, options(200, LineEnding::Lf)),
                vize_l0::cstr!("{{{{ {body} }}}}")
            );
            let source = vize_l0::cstr!("<template>{{{{{refused}}}}}</template>{script}");
            let arena = Allocator::default();
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let operand = original.first().unwrap();
            let syntax = operand.syntax();
            assert_eq!(
                syntax.hole(),
                None,
                "independent whole input remains within L1 admission"
            );
            assert!(syntax.admitted_expression().is_some());
            let root = core::ptr::from_ref(syntax.expression().unwrap());
            let refs = original.iter().collect::<std::vec::Vec<_>>();
            let Err(NativeTemplateRefusal::Expression {
                refusal: ExpressionRefusal::DepthLimit { span },
                ..
            }) = native_template_document(&owner, &refs, &arena)
            else {
                panic!("actual child at depth seventeen must refuse: {refused}")
            };
            assert_eq!(span.slice(syntax.source().text()), leaf);
            assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), root);
            assert_eq!(operand.content_span().slice(&source), operand.raw_content());
        }
        assert_preserved(&supported);
    }
}
