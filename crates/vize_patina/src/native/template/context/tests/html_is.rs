//! Equal bytes and ranges do not confer the actual callback's owner identity.
use super::*;
use crate::native::template::{NativeTemplateElement, NativeTemplateRule};
use crate::rules::vue::NoDeprecatedHtmlElementIs;
use crate::{RuleMeta, RuleRegistry};
use vize_l0::SmallVec;

fn view<'o, 'a>(root: &'o NativeLintComponent<'a>) -> NativeTemplateElement<'o, 'a> {
    let element = root.children().next().unwrap().into_element().unwrap();
    let mut attributes = SmallVec::new();
    for original in element.attributes() {
        let (checked, binding) = header::strict_binding(&element, original).unwrap();
        attributes.push(
            NativeTemplateAttribute::from_checked(
                checked,
                binding,
                NativeTemplateAttributeProfile::StaticOnly,
            )
            .unwrap(),
        );
    }
    NativeTemplateElement::new(element, attributes)
}

#[test]
fn html_is_callback_and_error_mint_refuse_equal_buffer_foreign_owners_even_when_clean() {
    let allocator = Allocator::new();
    for source in ["<div/>", "<div is=\"Other\"/>", "<div is=\"vue:Other\"/>"] {
        let block = SourceRoot::new(source).unwrap().whole_block();
        let owner = NativeLintComponent::parse_in(&allocator, block).unwrap();
        let foreign = NativeLintComponent::parse_in(&allocator, block).unwrap();
        assert!(core::ptr::eq(
            owner.component().block().source(),
            foreign.component().block().source()
        ));
        assert!(!core::ptr::eq(owner.component(), foreign.component()));
        let foreign_view = view(&foreign);
        let configured = Linter::new();
        let mut context = NativeTemplateLintContext::new(&configured, &owner, "Foreign.vue");
        context.current_rule = "vue/no-deprecated-html-element-is";
        assert_eq!(
            NoDeprecatedHtmlElementIs.run_on_template(&mut context, &foreign),
            Err(NativeTemplateLintRefusal::SourceMismatch)
        );
        assert_eq!(
            NoDeprecatedHtmlElementIs.run_on_element(&mut context, &foreign_view),
            Err(NativeTemplateLintRefusal::SourceMismatch)
        );
        for attribute in foreign_view.attributes() {
            assert_eq!(
                context.error_attribute_with_help(
                    attribute,
                    "vue/no-deprecated-html-element-is.message",
                    &[],
                    "vue/no-deprecated-html-element-is.help"
                ),
                Err(NativeTemplateLintRefusal::SourceMismatch)
            );
        }
        let expected = LintResult {
            filename: "Foreign.vue".into(),
            diagnostics: vec![],
            error_count: 0,
            warning_count: 0,
        };
        assert_eq!(format!("{:#?}", context.finish()), format!("{expected:#?}"));
    }
}

#[test]
fn matching_html_is_metadata_does_not_grant_the_actual_rule_capability() {
    struct Namesake;
    impl crate::Rule for Namesake {
        fn meta(&self) -> &'static RuleMeta {
            crate::Rule::meta(&NoDeprecatedHtmlElementIs)
        }
    }
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(Namesake));
    assert_eq!(
        Linter::with_registry(registry)
            .lint_native_template("<div is=\"MyComponent\" />", "Namesake.vue")
            .unwrap_err(),
        NativeTemplateLintRefusal::UnprovidedRule {
            rule: "vue/no-deprecated-html-element-is".into()
        }
    );
}
