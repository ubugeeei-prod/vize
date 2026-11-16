//! vue/valid-attribute-name
//!
//! Require valid attribute names in templates.
//!
//! Native-element attribute names and static `v-bind` arguments must be valid
//! XML names, matching eslint-plugin-vue's compatibility contract.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div my"attr="value"></div>
//! <div 0abc="value"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div my-attr="value"></div>
//! <div data-value="value"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupDirective, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/valid-attribute-name",
    description: "Require valid attribute names",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

#[derive(Default)]
/// Implements the `vue/valid-attribute-name` lint rule.
pub struct ValidAttributeName;

impl ValidAttributeName {
    /// XML 1.0 `Name`, as used by eslint-plugin-vue through
    /// `xml-name-validator`.
    fn is_valid_attribute_name(name: &str) -> bool {
        let mut chars = name.chars();
        chars.next().is_some_and(is_xml_name_start) && chars.all(is_xml_name_continue)
    }

    fn check(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        if element.is_component() {
            return;
        }
        element.walk_attributes(&mut |attribute| {
            let name = attribute.name();
            if !matches!(name.as_bytes().first(), Some(b'@' | b':' | b'#'))
                && !Self::is_valid_attribute_name(name)
            {
                ctx.error_at_with_help(
                    ctx.t_fmt("vue/valid-attribute-name.message", &[("name", name)]),
                    attribute.name_range(),
                    ctx.t("vue/valid-attribute-name.help"),
                );
            }
        });
        element.walk_directives(&mut |directive: MarkupDirective<'_>| {
            if !directive.name_eq("bind") {
                return;
            }
            let Some(name) = directive.arg_name() else {
                return;
            };
            if directive.is_static_arg_exact(name) && !Self::is_valid_attribute_name(name) {
                ctx.error_at_with_help(
                    ctx.t_fmt("vue/valid-attribute-name.message", &[("name", name)]),
                    directive.range(),
                    ctx.t("vue/valid-attribute-name.help"),
                );
            }
        });
    }
}

impl MarkupRule for ValidAttributeName {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check(ctx.lint(), element);
    }
}

fn is_xml_name_start(ch: char) -> bool {
    matches!(
        ch,
        ':' | 'A'..='Z' | '_' | 'a'..='z'
            | '\u{00C0}'..='\u{00D6}'
            | '\u{00D8}'..='\u{00F6}'
            | '\u{00F8}'..='\u{02FF}'
            | '\u{0370}'..='\u{037D}'
            | '\u{037F}'..='\u{1FFF}'
            | '\u{200C}'..='\u{200D}'
            | '\u{2070}'..='\u{218F}'
            | '\u{2C00}'..='\u{2FEF}'
            | '\u{3001}'..='\u{D7FF}'
            | '\u{F900}'..='\u{FDCF}'
            | '\u{FDF0}'..='\u{FFFD}'
            | '\u{10000}'..='\u{EFFFF}'
    )
}

fn is_xml_name_continue(ch: char) -> bool {
    is_xml_name_start(ch)
        || matches!(
            ch,
            '-' | '.' | '0'..='9' | '\u{00B7}' | '\u{0300}'..='\u{036F}' | '\u{203F}'..='\u{2040}'
        )
}

impl Rule for ValidAttributeName {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    // Invalid JSX attribute syntax exists only through parser recovery. Keep
    // that recovery-owned shape in the lowered lane instead of charging the
    // direct projection for a check it cannot authoritatively decide.
    fn jsx_needs_lowering(&self) -> bool {
        true
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check(ctx, &MarkupElement::new(element));
    }
}

#[cfg(test)]
mod tests {
    use super::ValidAttributeName;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(ValidAttributeName));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_normal_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div class="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_data_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div data-value="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_aria_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div aria-label="foo"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_static_bind_argument_on_native_element() {
        let result = create_linter().lint_template(r#"<slot :el$="el$" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
        assert_eq!(
            result.diagnostics[0].message,
            "Attribute name 'el$' is invalid"
        );
    }

    #[test]
    fn test_invalid_name_is_ignored_on_component() {
        let result = create_linter().lint_template(r#"<MyComponent :el$="el$" />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_name_on_native_element_with_is_attribute() {
        let result =
            create_linter().lint_template(r#"<button is="x-button" bad$="value" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
        assert_eq!(
            result.diagnostics[0].message,
            "Attribute name 'bad$' is invalid"
        );
    }
}
