//! Whole selected Call refusals preserve all original inputs and owners.

use super::super::{operands, selected};
use super::SCRIPTS;
use vize_glyph::native_doc::{ExpressionRefusal, NativeTemplateRefusal, native_template_document};
use vize_l0::Allocator;

#[test]
fn unsupported_calls_and_descendants_refuse_the_complete_selected_document() {
    for (content, ts_only) in [
        ("f?.(a)", false),
        ("f(a?.b)", false),
        ("(f?.x)(a)", false),
        ("f(...a)", false),
        ("new f(a)", false),
        ("f(new X)", false),
        ("import('x')", false),
        ("f(import('x'))", false),
        ("this.f(a)", false),
        ("f(this)", false),
        ("super.f(a)", false),
        ("super(a)", false),
        ("f(()=>a)", false),
        ("f({x:1})", false),
        ("f(&#91;...a&#93;)", false),
        ("f(a=b)", false),
        ("f(a?b:[...c])", false),
        ("f(a++)", false),
        ("f<T>(a)", true),
        ("(f as T)(a)", true),
        ("f(a as T)", true),
        ("f(a!)", true),
    ] {
        for script in SCRIPTS {
            if ts_only && script.is_empty() {
                continue;
            }
            let arena = Allocator::default();
            let source = vize_l0::cstr!(
                "<template>{{{{f(a)}}}}<p>{{{{/*原*/ {content}}}}}</p></template>{script}"
            );
            let owner = selected(&arena, &source);
            let original = operands(&owner);
            let syntax = original.get(1).unwrap().syntax();
            assert!(syntax.admitted_expression().is_some(), "{source}");
            let ast = core::ptr::from_ref(syntax.expression().unwrap());
            let comments = syntax
                .comments()
                .map(|comment| comment.text().unwrap().as_ptr())
                .collect::<std::vec::Vec<_>>();
            let map = syntax.source().decode_map().map(|map| map.segments());
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
                    .map(|comment| comment.text().unwrap().as_ptr())
                    .collect::<std::vec::Vec<_>>(),
                comments
            );
            assert_eq!(
                syntax
                    .source()
                    .decode_map()
                    .map(|map| map.segments().as_ptr()),
                map.map(|segments| segments.as_ptr())
            );
            for operand in &original {
                assert_eq!(operand.content_span().slice(&source), operand.raw_content());
            }
        }
    }
}

#[test]
fn genuine_call_syntax_holes_and_foreign_or_misordered_operands_never_admit() {
    for content in ["f(a +)", "f(,)", "f(...)"] {
        let arena = Allocator::default();
        let source =
            vize_l0::cstr!("<template>{{{{f(a)}}}}<p>{{{{/*kept*/ {content}}}}}</p></template>");
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
    let arena = Allocator::default();
    let source = "<template>{{f(a)}}<p>{{g(b)}}</p></template>";
    let owner = selected(&arena, source);
    let original = operands(&owner);
    let a = original.first().unwrap();
    let b = original.get(1).unwrap();
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
    assert_eq!(a.raw_content(), "f(a)");
    assert_eq!(b.raw_content(), "g(b)");
}
