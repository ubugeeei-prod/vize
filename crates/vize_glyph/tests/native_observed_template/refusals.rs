//! A failed build retains the actual observed prefix and complete selected source.

use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{
    ExpressionRefusal, NativeTemplateRefusal, ObservedNativeTemplateRefusal, TemplateRefusal,
    UnsupportedSyntax, observed_native_template_document,
};
use vize_l0::{Allocator, Span};
use vize_l1::SurfaceChild;
use vize_l1::markup::NativeInterpolationError;

use super::{SCRIPTS, selected};

#[test]
fn late_mapped_unsupported_current_is_owned_in_the_prefix_and_later_source_is_unobserved() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!(
            "<template>{{{{1n}}}}<p>{{{{a&#43;f(...b)}}}}</p>{{{{later}}}}</template>{script}"
        );
        let owner = selected(&arena, &source);
        let failure = observed_native_template_document(&owner, &arena).unwrap_err();
        assert!(core::ptr::eq(failure.original(), &owner));
        assert_eq!(failure.operands().len(), 2);
        assert_eq!(failure.operands()[0].raw_content(), "1n");
        let current = &failure.operands()[1];
        assert_eq!(current.raw_content(), "a&#43;f(...b)");
        assert_eq!(current.syntax().hole(), None);
        assert!(current.syntax().admitted_expression().is_some());
        let Expression::BinaryExpression(binary) = current.syntax().expression().unwrap() else {
            panic!("actual mapped Binary")
        };
        let Expression::CallExpression(call) = &binary.right else {
            panic!("actual unsupported Call")
        };
        let span = current.syntax().decoded_span(call.span).unwrap();
        assert_eq!(span, Span::new(2, 10));
        assert_eq!(
            failure.refusal(),
            ObservedNativeTemplateRefusal::Document {
                index: 1,
                refusal: NativeTemplateRefusal::Expression {
                    offset: 9,
                    refusal: ExpressionRefusal::UnsupportedNode { span },
                },
            }
        );
        assert!(failure.interpolation_failure().is_none());
        let root = core::ptr::from_ref(current.syntax().expression().unwrap());
        let view = current.syntax().source();
        let map = view.decode_map().unwrap().segments();
        let map_values = map.to_vec();
        assert_eq!(view.text(), "a+f(...b)");
        assert!(core::ptr::eq(view.authored_root(), source.as_str()));
        assert_eq!(
            owner.component().block().source(),
            "{{1n}}<p>{{a&#43;f(...b)}}</p>{{later}}"
        );
        let SurfaceChild::Interpolation(later) = owner.children().nth(2).unwrap().surface() else {
            panic!("untouched later source child")
        };
        assert_eq!(later.content.text, "later");
        let (same, mut prefix, refusal, unexpected) = failure.into_parts();
        prefix.reserve(32);
        assert!(core::ptr::eq(same, &owner));
        assert!(unexpected.is_none());
        assert!(matches!(
            refusal,
            ObservedNativeTemplateRefusal::Document { index: 1, .. }
        ));
        assert_eq!(prefix.len(), 2);
        assert_eq!(
            core::ptr::from_ref(prefix[1].syntax().expression().unwrap()),
            root
        );
        assert!(core::ptr::eq(
            prefix[1].syntax().source().text(),
            view.text()
        ));
        assert!(core::ptr::eq(
            prefix[1].syntax().source().decode_map().unwrap().segments(),
            map
        ));
        assert_eq!(
            prefix[1].syntax().source().decode_map().unwrap().segments(),
            map_values.as_slice()
        );
        assert_eq!(
            prefix[1].content_span().slice(&source),
            prefix[1].raw_content()
        );
        assert_eq!(prefix[1].syntax().diagnostics().count(), 0);
    }
}

#[test]
fn late_syntax_hole_keeps_rejected_current_comments_diagnostics_and_full_source() {
    for script in SCRIPTS {
        let arena = Allocator::default();
        let source = vize_l0::cstr!(
            "<template>{{{{1n}}}}<p>{{{{/*x*/ a+}}}}</p>{{{{later}}}}</template>{script}"
        );
        let owner = selected(&arena, &source);
        let failure = observed_native_template_document(&owner, &arena).unwrap_err();
        assert_eq!(failure.operands().len(), 2);
        let current = &failure.operands()[1];
        let syntax = current.syntax();
        let hole = syntax.hole();
        assert!(hole.is_some());
        assert!(syntax.expression().is_none());
        assert_eq!(syntax.comments().count(), 1);
        assert_eq!(syntax.comments().next().unwrap().text().unwrap(), "/*x*/");
        assert!(syntax.diagnostics().count() > 0);
        assert_eq!(
            failure.refusal(),
            ObservedNativeTemplateRefusal::Document {
                index: 1,
                refusal: NativeTemplateRefusal::OperandRejected {
                    offset: 9,
                    index: 1,
                    hole
                },
            }
        );
        assert!(failure.interpolation_failure().is_none());
        let text = syntax.source().text().as_ptr();
        let diagnostics = syntax.diagnostics().count();
        let comments = syntax
            .comments()
            .map(|comment| {
                (
                    comment.kind(),
                    comment.authored_span().unwrap(),
                    comment.text().unwrap().as_ptr(),
                )
            })
            .collect::<std::vec::Vec<_>>();
        let (same, prefix, _, unexpected) = failure.into_parts();
        assert!(core::ptr::eq(same, &owner));
        assert!(unexpected.is_none());
        assert_eq!(prefix.len(), 2);
        assert_eq!(prefix[1].syntax().hole(), hole);
        assert_eq!(prefix[1].syntax().source().text().as_ptr(), text);
        assert_eq!(prefix[1].syntax().diagnostics().count(), diagnostics);
        assert_eq!(
            prefix[1]
                .syntax()
                .comments()
                .map(|comment| (
                    comment.kind(),
                    comment.authored_span().unwrap(),
                    comment.text().unwrap().as_ptr()
                ))
                .collect::<std::vec::Vec<_>>(),
            comments
        );
        assert_eq!(prefix[1].raw_content(), "/*x*/ a+");
        assert!(core::ptr::eq(
            prefix[1].syntax().source().authored_root(),
            source.as_str()
        ));
        assert_eq!(
            owner.component().block().source(),
            "{{1n}}<p>{{/*x*/ a+}}</p>{{later}}"
        );
    }
}

#[test]
fn genuine_observation_failure_is_kept_without_inventing_a_current_operand() {
    let arena = Allocator::default();
    let source = "<template>{{1n}}<p>{{b}}</template>";
    let owner = selected(&arena, source);
    assert!(owner.component().carrier().errors.is_empty());
    let failure = observed_native_template_document(&owner, &arena).unwrap_err();
    assert_eq!(failure.operands().len(), 1);
    assert_eq!(failure.operands()[0].raw_content(), "1n");
    assert_eq!(
        failure.refusal(),
        ObservedNativeTemplateRefusal::Interpolation {
            offset: 9,
            index: 1,
            kind: NativeInterpolationError::RecoveredComponent,
        }
    );
    let actual = failure.interpolation_failure().unwrap();
    assert_eq!(actual.kind(), NativeInterpolationError::RecoveredComponent);
    assert!(actual.syntax().is_none());
    let root = core::ptr::from_ref(failure.operands()[0].syntax().expression().unwrap());
    let (same, prefix, refusal, unexpected) = failure.into_parts();
    assert!(core::ptr::eq(same, &owner));
    assert_eq!(prefix.len(), 1);
    assert_eq!(
        core::ptr::from_ref(prefix[0].syntax().expression().unwrap()),
        root
    );
    assert!(matches!(
        refusal,
        ObservedNativeTemplateRefusal::Interpolation { index: 1, .. }
    ));
    let actual = unexpected.unwrap();
    assert_eq!(actual.kind(), NativeInterpolationError::RecoveredComponent);
    assert!(actual.syntax().is_none());
    assert_eq!(same.component().block().source(), "{{1n}}<p>{{b}}");
    assert!(core::ptr::eq(
        same.component().block().root_source(),
        source
    ));
}

#[test]
fn earlier_tag_refusal_stops_before_observing_its_body_and_retains_completed_prefix() {
    let arena = Allocator::default();
    let source = "<template>{{1n}}<p v->{{later}}</p></template>";
    let owner = selected(&arena, source);
    let failure = observed_native_template_document(&owner, &arena).unwrap_err();
    assert_eq!(failure.operands().len(), 1);
    assert_eq!(failure.operands()[0].raw_content(), "1n");
    assert!(matches!(
        failure.refusal(),
        ObservedNativeTemplateRefusal::Document {
            index: 1,
            refusal: NativeTemplateRefusal::Template(TemplateRefusal::Unsupported {
                syntax: UnsupportedSyntax::Directive,
                ..
            }),
        }
    ));
    assert!(failure.interpolation_failure().is_none());
    assert_eq!(
        failure.original().component().block().source(),
        "{{1n}}<p v->{{later}}</p>"
    );
    assert!(core::ptr::eq(
        failure.original().component().block().root_source(),
        source
    ));
}
