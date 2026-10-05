use super::*;
use crate::{
    native::template::{NativeTemplateElement, NativeTemplateRule},
    rules::a11y::{NoAccessKey, NoAutofocus},
};
use vize_l0::SmallVec;

fn view<'o, 'a>(root: &'o NativeLintComponent<'a>) -> NativeTemplateElement<'o, 'a> {
    let element = root.children().next().unwrap().into_element().unwrap();
    let mut attributes = SmallVec::new();
    for original in element.attributes() {
        let (checked, binding) = header::wide_binding(&element, original).unwrap();
        attributes.push(
            NativeTemplateAttribute::from_checked(
                checked,
                binding,
                NativeTemplateAttributeProfile::Bindings,
            )
            .unwrap(),
        );
    }
    NativeTemplateElement::new(element, attributes)
}

fn empty() -> LintResult {
    LintResult {
        filename: "exact.vue".into(),
        diagnostics: vec![],
        error_count: 0,
        warning_count: 0,
    }
}

#[test]
fn concrete_focus_root_and_element_callbacks_refuse_same_buffer_foreign_owner_even_before_zero_findings()
 {
    let allocator = Allocator::new();
    for source in [
        "<div/>",
        "<div :[autofocus]='opaque' .[accesskey]='opaque'/>",
        "<div autofocus accesskey='h'/>",
    ] {
        let block = SourceRoot::new(source).unwrap().whole_block();
        let original = NativeLintComponent::parse_in(&allocator, block).unwrap();
        let foreign = NativeLintComponent::parse_in(&allocator, block).unwrap();
        let foreign_view = view(&foreign);
        let configured = Linter::new();
        for (rule, callback) in [
            ("a11y/no-autofocus", &NoAutofocus as &dyn NativeTemplateRule),
            (
                "a11y/no-access-key",
                &NoAccessKey as &dyn NativeTemplateRule,
            ),
        ] {
            let mut context = NativeTemplateLintContext::new(&configured, &original, "exact.vue");
            context.current_rule = rule;
            assert_eq!(
                callback.run_on_template(&mut context, &foreign),
                Err(NativeTemplateLintRefusal::SourceMismatch)
            );
            assert_eq!(
                callback.run_on_element(&mut context, &foreign_view),
                Err(NativeTemplateLintRefusal::SourceMismatch)
            );
            assert_eq!(
                format!("{:#?}", context.finish()),
                format!("{:#?}", empty())
            );
        }
    }
}

#[test]
fn authentic_same_owner_proofs_keep_full_physical_attribute_range_and_empty_root_output() {
    let allocator = Allocator::new();
    for (source, rule, callback, message, help, end) in [
        (
            "<div autofocus/>",
            "a11y/no-autofocus",
            &NoAutofocus as &dyn NativeTemplateRule,
            "The autofocus attribute should not be used. It can disrupt navigation for screen reader users",
            "Remove the autofocus attribute. Manage focus programmatically when needed using ref and focus()",
            14,
        ),
        (
            "<div accesskey='h'/>",
            "a11y/no-access-key",
            &NoAccessKey as &dyn NativeTemplateRule,
            "The accesskey attribute should not be used. Access keys create keyboard shortcut conflicts",
            "Remove the accesskey attribute. Access keys create inconsistent keyboard shortcuts across platforms and conflict with assistive technology shortcuts",
            18,
        ),
    ] {
        let root = NativeLintComponent::parse_in(
            &allocator,
            SourceRoot::new(source).unwrap().whole_block(),
        )
        .unwrap();
        let element = view(&root);
        let configured = Linter::new();
        let mut context = NativeTemplateLintContext::new(&configured, &root, "exact.vue");
        context.current_rule = rule;
        assert_eq!(callback.run_on_template(&mut context, &root), Ok(()));
        assert_eq!(callback.run_on_element(&mut context, &element), Ok(()));
        let expected = LintResult {
            filename: "exact.vue".into(),
            error_count: 0,
            warning_count: 1,
            diagnostics: vec![LintDiagnostic {
                rule_name: rule,
                severity: Severity::Warning,
                message: message.into(),
                start: 5,
                end,
                help: Some(help.into()),
                labels: vec![],
                fix: None,
            }],
        };
        assert_eq!(format!("{:#?}", context.finish()), format!("{expected:#?}"));
    }
}
