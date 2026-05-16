//! vue/no-duplicate-attributes
//!
//! Disallow duplicate attributes on the same element.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div id="foo" id="bar"></div>
//! <div :class="foo" class="bar"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div id="foo"></div>
//! <div :class="foo"></div>
//! ```

#![allow(clippy::disallowed_macros)]

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_relief::ast::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-duplicate-attributes",
    description: "Disallow duplicate attributes on the same element",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow duplicate attributes
pub struct NoDuplicateAttributes {
    /// Allow :class and class to coexist
    pub allow_coexist_class: bool,
    /// Allow :style and style to coexist
    pub allow_coexist_style: bool,
}

impl Default for NoDuplicateAttributes {
    fn default() -> Self {
        Self {
            allow_coexist_class: true,
            allow_coexist_style: true,
        }
    }
}

impl Rule for NoDuplicateAttributes {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let mut seen_attrs: FxHashSet<String> = FxHashSet::default();
        let mut seen_directives: FxHashSet<String> = FxHashSet::default();

        for prop in element.props.iter() {
            match prop {
                PropNode::Attribute(attr) => {
                    let name = attr.name.as_str().to_lowercase();

                    // Check for duplicate static attributes
                    if seen_attrs.contains(name.as_str()) {
                        ctx.error_with_help(
                            ctx.t_fmt("vue/no-duplicate-attributes.message", &[("attr", &name)]),
                            &attr.loc,
                            ctx.t("vue/no-duplicate-attributes.help"),
                        );
                    } else {
                        seen_attrs.insert(name.clone().into());
                    }

                    // Check for coexistence with directives (unless allowed)
                    if !self.allow_coexist_class
                        && name == "class"
                        && seen_directives.contains("class")
                    {
                        ctx.error_with_help(
                            ctx.t_fmt("vue/no-duplicate-attributes.message", &[("attr", "class")]),
                            &attr.loc,
                            ctx.t("vue/no-duplicate-attributes.help"),
                        );
                    }
                    if !self.allow_coexist_style
                        && name == "style"
                        && seen_directives.contains("style")
                    {
                        ctx.error_with_help(
                            ctx.t_fmt("vue/no-duplicate-attributes.message", &[("attr", "style")]),
                            &attr.loc,
                            ctx.t("vue/no-duplicate-attributes.help"),
                        );
                    }
                }
                PropNode::Directive(dir) => {
                    // Handle v-bind directives
                    if dir.name.as_str() == "bind" {
                        if let Some(ref arg) = dir.arg {
                            let arg_name = get_expression_content(arg).to_lowercase();
                            let arg_name_str = arg_name.as_str();

                            // Check for duplicate directives
                            if seen_directives.contains(arg_name_str) {
                                ctx.error_with_help(
                                    ctx.t_fmt(
                                        "vue/no-duplicate-attributes.message",
                                        &[("attr", &format!("v-bind:{}", arg_name))],
                                    ),
                                    &dir.loc,
                                    ctx.t("vue/no-duplicate-attributes.help"),
                                );
                            } else {
                                seen_directives.insert(arg_name_str.into());
                            }

                            // Check for coexistence with static attributes (unless allowed)
                            if !self.allow_coexist_class
                                && arg_name == "class"
                                && seen_attrs.contains("class")
                            {
                                ctx.error_with_help(
                                    ctx.t_fmt(
                                        "vue/no-duplicate-attributes.message",
                                        &[("attr", "v-bind:class")],
                                    ),
                                    &dir.loc,
                                    ctx.t("vue/no-duplicate-attributes.help"),
                                );
                            }
                            if !self.allow_coexist_style
                                && arg_name == "style"
                                && seen_attrs.contains("style")
                            {
                                ctx.error_with_help(
                                    ctx.t_fmt(
                                        "vue/no-duplicate-attributes.message",
                                        &[("attr", "v-bind:style")],
                                    ),
                                    &dir.loc,
                                    ctx.t("vue/no-duplicate-attributes.help"),
                                );
                            }
                        }
                    }
                    // Handle v-on directives
                    else if dir.name.as_str() == "on" {
                        if let Some(ref arg) = dir.arg {
                            let event_name = get_expression_content(arg);
                            // Include modifiers in the key to allow @keydown.left and @keydown.right
                            let modifiers: Vec<&str> =
                                dir.modifiers.iter().map(|m| m.content.as_str()).collect();
                            let event_key = if modifiers.is_empty() {
                                format!("on:{}", event_name)
                            } else {
                                format!("on:{}.{}", event_name, modifiers.join("."))
                            };
                            if seen_directives.contains(event_key.as_str()) {
                                let display_name = if modifiers.is_empty() {
                                    format!("v-on:{}", event_name)
                                } else {
                                    format!("v-on:{}.{}", event_name, modifiers.join("."))
                                };
                                ctx.error_with_help(
                                    ctx.t_fmt(
                                        "vue/no-duplicate-attributes.message",
                                        &[("attr", &display_name)],
                                    ),
                                    &dir.loc,
                                    ctx.t("vue/no-duplicate-attributes.help"),
                                );
                            } else {
                                seen_directives.insert(event_key.into());
                            }
                        }
                    }
                    // Handle v-model
                    else if dir.name.as_str() == "model" {
                        let model_key = if let Some(ref arg) = dir.arg {
                            format!("model:{}", get_expression_content(arg))
                        } else {
                            "model:modelValue".to_owned()
                        };
                        if seen_directives.contains(model_key.as_str()) {
                            ctx.error_with_help(
                                ctx.t_fmt(
                                    "vue/no-duplicate-attributes.message",
                                    &[("attr", "v-model")],
                                ),
                                &dir.loc,
                                ctx.t("vue/no-duplicate-attributes.help"),
                            );
                        } else {
                            seen_directives.insert(model_key.into());
                        }
                    }
                }
            }
        }
    }
}

/// Get content from ExpressionNode
fn get_expression_content(expr: &vize_relief::ast::ExpressionNode) -> String {
    match expr {
        vize_relief::ast::ExpressionNode::Simple(s) => s.content.to_compact_string(),
        vize_relief::ast::ExpressionNode::Compound(_) => "<dynamic>".to_compact_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::NoDuplicateAttributes;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoDuplicateAttributes::default()));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_unique_attributes() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div id="foo" class="bar"></div>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_id() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div id="foo" id="bar"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_valid_class_coexist() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="foo" class="bar"></div>"#, "test.vue");
        // Default allows coexistence
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_v_bind() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :id="foo" :id="bar"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_different_event_modifiers() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div @keydown.left="goLeft" @keydown.right="goRight"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_different_event_modifiers_multiple() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div @click.stop="a" @click.prevent="b" @click.stop.prevent="c"></div>"#,
            "test.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_duplicate_event_same_modifiers() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<div @click.stop="a" @click.stop="b"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_duplicate_event_no_modifiers() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div @click="a" @click="b"></div>"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }
}
