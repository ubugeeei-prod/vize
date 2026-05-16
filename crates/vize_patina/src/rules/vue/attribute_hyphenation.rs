//! vue/attribute-hyphenation
//!
//! Enforce attribute naming style on custom components.
//!
//! ## Examples
//!
//! ### Invalid (default: always)
//! ```vue
//! <MyComponent myProp="value" />
//! <MyComponent :myProp="value" />
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent my-prop="value" />
//! <MyComponent :my-prop="value" />
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_croquis::naming::is_camel_case;
use vize_relief::ast::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/attribute-hyphenation",
    description: "Enforce attribute naming style on custom components",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Attribute hyphenation style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HyphenationStyle {
    /// Require hyphenated attribute names: my-prop
    #[default]
    Always,
    /// Allow camelCase: myProp
    Never,
}

/// Attribute hyphenation rule
pub struct AttributeHyphenation {
    pub style: HyphenationStyle,
    /// Attributes to ignore
    pub ignore: Vec<String>,
}

impl Default for AttributeHyphenation {
    fn default() -> Self {
        Self {
            style: HyphenationStyle::Always,
            ignore: vec![
                // Common data attributes
                "data-".to_compact_string(),
                "aria-".to_compact_string(),
                // Vue specific
                "slot-scope".to_compact_string(),
            ],
        }
    }
}

impl AttributeHyphenation {
    fn is_custom_component(tag: &str) -> bool {
        // Custom components are either:
        // 1. PascalCase (starts with uppercase)
        // 2. Contains hyphen (kebab-case component)
        // 3. Not a known HTML element
        if tag.chars().next().is_some_and(|c| c.is_uppercase()) {
            return true;
        }
        if tag.contains('-') {
            return true;
        }
        false
    }

    fn should_ignore(&self, name: &str) -> bool {
        for pattern in &self.ignore {
            if pattern.ends_with('-') {
                // Prefix pattern
                if name.starts_with(pattern.as_str()) {
                    return true;
                }
            } else if name == pattern {
                return true;
            }
        }
        false
    }
}

impl Rule for AttributeHyphenation {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let tag = element.tag.as_str();

        // Only check custom components
        if !Self::is_custom_component(tag) {
            return;
        }

        for prop in &element.props {
            let (name, loc) = match prop {
                PropNode::Attribute(attr) => (attr.name.as_str(), &attr.loc),
                PropNode::Directive(dir) => {
                    // Check v-bind argument (:my-prop)
                    if dir.name.as_str() == "bind" {
                        if let Some(arg) = &dir.arg {
                            match arg {
                                vize_relief::ast::ExpressionNode::Simple(s) => {
                                    (s.content.as_str(), &dir.loc)
                                }
                                _ => continue,
                            }
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
            };

            // Skip ignored attributes
            if self.should_ignore(name) {
                continue;
            }

            // Skip v-* directives, @ events, # slots
            if name.starts_with("v-")
                || name.starts_with('@')
                || name.starts_with('#')
                || name.starts_with("on")
            {
                continue;
            }

            match self.style {
                HyphenationStyle::Always => {
                    if is_camel_case(name) {
                        ctx.warn_with_help(
                            ctx.t("vue/attribute-hyphenation.message"),
                            loc,
                            ctx.t("vue/attribute-hyphenation.help"),
                        );
                    }
                }
                HyphenationStyle::Never => {
                    // In "never" mode, we don't require kebab-case
                    // (but this mode is rarely used)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AttributeHyphenation;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(AttributeHyphenation::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_hyphenated() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent my-prop="value" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_camel_case() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent myProp="value" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_html_element() {
        let linter = create_linter();
        // HTML elements don't require hyphenation
        let result = linter.lint_template(r#"<div onClick="handler"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_data_attribute() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<MyComponent data-testId="123" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
