//! vue/v-bind-style
//!
//! Enforce `v-bind` directive style.
//!
//! ## Options
//!
//! - `"shorthand"` (default): Prefer `:attr` over `v-bind:attr`
//! - `"longform"`: Prefer `v-bind:attr` over `:attr`
//!
//! ## Examples
//!
//! ### Invalid (with shorthand option)
//! ```vue
//! <div v-bind:class="foo"></div>
//! ```
//!
//! ### Valid (with shorthand option)
//! ```vue
//! <div :class="foo"></div>
//! ```

#![allow(clippy::disallowed_macros)]

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_relief::ast::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/v-bind-style",
    description: "Enforce `v-bind` directive style",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Style preference for v-bind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VBindStyleOption {
    #[default]
    Shorthand,
    Longform,
}

/// Enforce v-bind directive style
pub struct VBindStyle {
    pub style: VBindStyleOption,
}

impl Default for VBindStyle {
    fn default() -> Self {
        Self {
            style: VBindStyleOption::Shorthand,
        }
    }
}

impl Rule for VBindStyle {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "bind" {
            return;
        }

        // Skip object binding syntax (v-bind="...")
        if directive.arg.is_none() {
            return;
        }

        let raw_name = directive.raw_name.as_deref().unwrap_or("");
        let is_shorthand = raw_name.starts_with(':');

        match self.style {
            VBindStyleOption::Shorthand => {
                if !is_shorthand {
                    // Using v-bind:attr, should use :attr
                    let arg_content = directive
                        .arg
                        .as_ref()
                        .map(|a| match a {
                            vize_relief::ast::ExpressionNode::Simple(s) => {
                                s.content.to_compact_string()
                            }
                            _ => String::default(),
                        })
                        .unwrap_or_default();

                    let exp_content = directive
                        .exp
                        .as_ref()
                        .map(|e| match e {
                            vize_relief::ast::ExpressionNode::Simple(s) => {
                                s.content.to_compact_string()
                            }
                            _ => String::default(),
                        })
                        .unwrap_or_default();

                    let new_text = format!(":{}=\"{}\"", arg_content, exp_content);
                    let fix = Fix::new(
                        "Use shorthand syntax",
                        TextEdit::replace(
                            directive.loc.start.offset,
                            directive.loc.end.offset,
                            new_text,
                        ),
                    );

                    ctx.report(
                        LintDiagnostic::warn(
                            META.name,
                            "Prefer shorthand `:` over `v-bind:`",
                            directive.loc.start.offset,
                            directive.loc.end.offset,
                        )
                        .with_help("Use `:attr=\"value\"` instead of `v-bind:attr=\"value\"`")
                        .with_fix(fix),
                    );
                }
            }
            VBindStyleOption::Longform => {
                if is_shorthand {
                    // Using :attr, should use v-bind:attr
                    let arg_content = directive
                        .arg
                        .as_ref()
                        .map(|a| match a {
                            vize_relief::ast::ExpressionNode::Simple(s) => {
                                s.content.to_compact_string()
                            }
                            _ => String::default(),
                        })
                        .unwrap_or_default();

                    let exp_content = directive
                        .exp
                        .as_ref()
                        .map(|e| match e {
                            vize_relief::ast::ExpressionNode::Simple(s) => {
                                s.content.to_compact_string()
                            }
                            _ => String::default(),
                        })
                        .unwrap_or_default();

                    let new_text = format!("v-bind:{}=\"{}\"", arg_content, exp_content);
                    let fix = Fix::new(
                        "Use longform syntax",
                        TextEdit::replace(
                            directive.loc.start.offset,
                            directive.loc.end.offset,
                            new_text,
                        ),
                    );

                    ctx.report(
                        LintDiagnostic::warn(
                            META.name,
                            "Prefer `v-bind:` over shorthand `:`",
                            directive.loc.start.offset,
                            directive.loc.end.offset,
                        )
                        .with_help("Use `v-bind:attr=\"value\"` instead of `:attr=\"value\"`")
                        .with_fix(fix),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{VBindStyle, VBindStyleOption};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter_shorthand() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VBindStyle::default()));
        Linter::with_registry(registry)
    }

    fn create_linter_longform() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VBindStyle {
            style: VBindStyleOption::Longform,
        }));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_shorthand() {
        let linter = create_linter_shorthand();
        let result = linter.lint_template(r#"<div :class="foo"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_longform_with_shorthand_option() {
        let linter = create_linter_shorthand();
        let result = linter.lint_template(r#"<div v-bind:class="foo"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].has_fix());
    }

    #[test]
    fn test_valid_longform() {
        let linter = create_linter_longform();
        let result = linter.lint_template(r#"<div v-bind:class="foo"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_shorthand_with_longform_option() {
        let linter = create_linter_longform();
        let result = linter.lint_template(r#"<div :class="foo"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].has_fix());
    }
}
