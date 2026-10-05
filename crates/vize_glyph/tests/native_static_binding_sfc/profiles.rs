use super::{options, policy::empty_refusal, selected};
use oxc_ast::ast::Expression;
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeSfcBlockRole, NativeSfcRefusal, NativeTemplateRefusal,
    NativeTemplateValuePolicy, ObservedNativeTemplateRefusal, observe_native_sfc_in,
    observed_native_template_document_with_policy,
};
use vize_l0::{Allocator, Span};
use vize_l1::container::vue::ScriptRole;
use vize_l1::{embed::Lang, markup::NativeTemplateGrammar};

#[test]
fn genuine_setup_ts_profile_keeps_original_lower_binding_but_whole_sfc_refuses_its_script_role() {
    let source = "<template><p :id='value as boolean'/></template><script setup lang=ts>const value=true</script>";
    let arena = Allocator::default();
    let policy = options(200, 2, LineEnding::Lf);
    let owner = observe_native_sfc_in(&arena, source, policy);
    empty_refusal(
        &owner,
        source,
        policy,
        NativeSfcRefusal::UnsupportedBlock {
            index: 1,
            span: Span::new(48, 70),
            role: NativeSfcBlockRole::Script(ScriptRole::Setup),
        },
    );
    assert!(owner.selected().is_none());
    let selected = selected(&arena, source, policy);
    assert_eq!(selected.grammar(), NativeTemplateGrammar::TypeScriptModule);
    let failure = observed_native_template_document_with_policy(
        &selected,
        &arena,
        NativeTemplateValuePolicy::FormatConditionalsAndStaticBindings,
    )
    .unwrap_err();
    assert_eq!(
        failure.refusal(),
        ObservedNativeTemplateRefusal::Document {
            index: 0,
            refusal: NativeTemplateRefusal::BindingExpression {
                span: Span::new(18, 34),
                refusal: ExpressionRefusal::UnsupportedNode {
                    span: Span::new(0, 16)
                }
            }
        }
    );
    assert_eq!(failure.binding_operands().len(), 1);
    assert!(failure.binding_failure().is_none());
    let binding = &failure.binding_operands()[0];
    assert_eq!(binding.raw_value(), "value as boolean");
    assert_eq!(binding.syntax().source().text(), "value as boolean");
    assert_eq!(binding.syntax().grammar().lang, Lang::Ts);
    assert!(binding.syntax().source_type().is_typescript());
    assert_eq!(binding.syntax().hole(), None);
    assert!(matches!(
        binding.syntax().expression(),
        Some(Expression::TSAsExpression(_))
    ));
    let element = selected.children().next().unwrap().into_element().unwrap();
    assert!(
        binding
            .admitted_for(&selected, element.attributes().next().unwrap())
            .is_some()
    );
    assert_eq!(binding.syntax().diagnostics().count(), 0);
    assert_eq!(binding.syntax().comments().count(), 0);
}
