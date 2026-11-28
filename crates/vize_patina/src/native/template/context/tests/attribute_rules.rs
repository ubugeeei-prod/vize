//! New real attribute callbacks retain the existing original-owner boundary.

use super::*;
use crate::rule::Rule;
use crate::rules::html::DeprecatedAttr;
use crate::rules::opinionated::vue::NoInlineStyle;

#[test]
fn equal_buffer_foreign_roots_cannot_replace_the_actual_callback_owner() {
    let allocator = Allocator::new();
    let source = r#"<div style="red" align="center"/>"#;
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = ComponentTagAnalysis::parse_in(&allocator, block).unwrap();
    let foreign = ComponentTagAnalysis::parse_in(&allocator, block).unwrap();
    assert!(core::ptr::eq(
        original.component().block().source(),
        foreign.component().block().source(),
    ));
    assert!(!core::ptr::eq(original.component(), foreign.component()));
    let configured = Linter::new();
    let rules: &[&dyn Rule] = &[&NoInlineStyle, &DeprecatedAttr];
    for rule in rules {
        let mut context = NativeTemplateLintContext::new(&configured, &original, "Exact.vue");
        context.current_rule = rule.meta().name;
        let callback = rule.as_native_template_rule().unwrap();
        assert_eq!(
            callback.run_on_template(&mut context, &foreign),
            Err(NativeTemplateLintRefusal::SourceMismatch)
        );
        assert!(context.finish().diagnostics.is_empty());
    }
}

#[test]
fn formatted_help_needs_the_original_checked_attribute_and_keeps_the_complete_packet() {
    let allocator = Allocator::new();
    let source = r#"<div align="center"/>"#;
    let block = SourceRoot::new(source).unwrap().whole_block();
    let original = ComponentTagAnalysis::parse_in(&allocator, block).unwrap();
    let foreign = ComponentTagAnalysis::parse_in(&allocator, block).unwrap();
    let mut registry = crate::RuleRegistry::new();
    registry.register(Box::new(DeprecatedAttr));
    let configured = Linter::with_registry(registry)
        .with_rule_severity_overrides(vec![("html/deprecated-attr".into(), Severity::Error)]);
    let mut context = NativeTemplateLintContext::new(&configured, &original, "Exact.vue");
    context.current_rule = "html/deprecated-attr";
    for (owner, accepted) in [(&foreign, false), (&original, true)] {
        let element = owner.children().next().unwrap().into_element().unwrap();
        let attribute = element.attributes().next().unwrap();
        let (checked, binding) = header::wide_binding(&element, attribute).unwrap();
        let proof = NativeTemplateAttribute::from_checked(
            checked,
            binding,
            NativeTemplateAttributeProfile::Bindings,
        )
        .unwrap();
        let result = context.warn_attribute_with_formatted_help(
            &proof,
            "html/deprecated-attr.message",
            &[("attr", "align"), ("tag", "div")],
            "html/deprecated-attr.help",
            &[("suggestion", "CSS `text-align` or `margin: auto`")],
        );
        assert_eq!(
            result,
            if accepted {
                Ok(())
            } else {
                Err(NativeTemplateLintRefusal::SourceMismatch)
            }
        );
    }
    assert_eq!(
        format!("{:?}", context.finish()),
        format!("{:?}", configured.lint_template(source, "Exact.vue"))
    );
}
