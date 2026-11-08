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
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ElementType, ExpressionNode, PropNode};

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

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag_type == ElementType::Component {
            return;
        }
        for prop in &element.props {
            match prop {
                // Directive shorthand inside a v-pre subtree is deliberately
                // retained as a plain attribute by the template parser. The
                // upstream rule does not inspect that raw subtree.
                PropNode::Attribute(attr)
                    if !matches!(attr.name.as_bytes().first(), Some(b'@' | b':' | b'#'))
                        && !Self::is_valid_attribute_name(attr.name) =>
                {
                    ctx.error_with_help(
                        ctx.t_fmt("vue/valid-attribute-name.message", &[("name", attr.name)]),
                        &attr.name_loc,
                        ctx.t("vue/valid-attribute-name.help"),
                    );
                }
                PropNode::Directive(directive) if directive.name == "bind" => {
                    let Some(ExpressionNode::Simple(argument)) = directive.arg.as_ref() else {
                        continue;
                    };
                    if argument.is_static && !Self::is_valid_attribute_name(argument.content) {
                        ctx.error_with_help(
                            ctx.t_fmt(
                                "vue/valid-attribute-name.message",
                                &[("name", argument.content)],
                            ),
                            &directive.loc,
                            ctx.t("vue/valid-attribute-name.help"),
                        );
                    }
                }
                _ => {}
            }
        }
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
}
