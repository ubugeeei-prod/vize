//! vue/v-on-style
//!
//! Enforce `v-on` directive style.
//!
//! ## Options
//!
//! - `"shorthand"` (default): Prefer `@event` over `v-on:event`
//! - `"longform"`: Prefer `v-on:event` over `@event`
//!
//! ## Examples
//!
//! ### Invalid (with shorthand option)
//! ```vue
//! <div v-on:click="handleClick"></div>
//! ```
//!
//! ### Valid (with shorthand option)
//! ```vue
//! <div @click="handleClick"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::markup::{
    MarkupContext, MarkupDirective, MarkupElement, MarkupRule, MarkupTraversal,
};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::String;
use vize_relief::{DirectiveNode, ElementNode};

static META: RuleMeta = RuleMeta {
    name: "vue/v-on-style",
    description: "Enforce `v-on` directive style",
    category: RuleCategory::StronglyRecommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// Style preference for v-on
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VOnStyleOption {
    #[default]
    /// Require `@event` shorthand syntax.
    Shorthand,
    /// Require explicit `v-on:event` syntax.
    Longform,
}

/// Enforce v-on directive style
pub struct VOnStyle {
    /// Directive spelling accepted by the rule.
    pub style: VOnStyleOption,
}

impl VOnStyle {
    fn check(&self, ctx: &mut LintContext<'_>, directive: &MarkupDirective<'_>) {
        if !directive.name_eq("on") || directive.argument_range().is_none() {
            return;
        }

        let range = directive.range();
        let Some(attribute) = ctx.source.get(range.start as usize..range.end as usize) else {
            return;
        };
        let is_shorthand = attribute.starts_with('@');
        let should_report = match self.style {
            VOnStyleOption::Shorthand => !is_shorthand,
            VOnStyleOption::Longform => is_shorthand,
        };
        if !should_report {
            return;
        }
        let Some(new_text) = replacement_text(attribute, self.style) else {
            return;
        };
        let (message, help, fix_message) = match self.style {
            VOnStyleOption::Shorthand => (
                "Prefer shorthand `@` over `v-on:`",
                "Use `@event=\"handler\"` instead of `v-on:event=\"handler\"`",
                "Use shorthand syntax",
            ),
            VOnStyleOption::Longform => (
                "Prefer `v-on:` over shorthand `@`",
                "Use `v-on:event=\"handler\"` instead of `@event=\"handler\"`",
                "Use longform syntax",
            ),
        };
        ctx.report(
            LintDiagnostic::warn(META.name, message, range.start, range.end)
                .with_help(help)
                .with_fix(Fix::new(
                    fix_message,
                    TextEdit::replace(range.start, range.end, new_text),
                )),
        );
    }
}

impl MarkupRule for VOnStyle {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Directives
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        _element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        self.check(ctx.lint(), directive);
    }
}

impl Default for VOnStyle {
    fn default() -> Self {
        Self {
            style: VOnStyleOption::Shorthand,
        }
    }
}

impl Rule for VOnStyle {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        self.check(ctx, &MarkupDirective::from_relief(directive));
    }
}

fn replacement_text(attribute: &str, target: VOnStyleOption) -> Option<String> {
    match target {
        VOnStyleOption::Shorthand => {
            let rest = attribute.strip_prefix("v-on:")?;
            let mut new_text = String::with_capacity(1 + rest.len());
            new_text.push('@');
            new_text.push_str(rest);
            Some(new_text)
        }
        VOnStyleOption::Longform => {
            let rest = attribute.strip_prefix('@')?;
            let mut new_text = String::with_capacity(5 + rest.len());
            new_text.push_str("v-on:");
            new_text.push_str(rest);
            Some(new_text)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{VOnStyle, VOnStyleOption};
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter(style: VOnStyleOption) -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VOnStyle { style }));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_shorthand() {
        let linter = create_linter(VOnStyleOption::Shorthand);
        let result = linter.lint_template(r#"<div @click="handleClick"></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_longform_with_shorthand_option() {
        let linter = create_linter(VOnStyleOption::Shorthand);
        let source = r#"<div v-on:click="handleClick"></div>"#;
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].has_fix());
        assert_eq!(
            result.diagnostics[0]
                .fix
                .as_ref()
                .unwrap()
                .apply(source)
                .as_str(),
            r#"<div @click="handleClick"></div>"#
        );
    }

    #[test]
    fn test_invalid_shorthand_with_longform_option() {
        let linter = create_linter(VOnStyleOption::Longform);
        let source = r#"<div @click="handleClick"></div>"#;
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].has_fix());
        assert_eq!(
            result.diagnostics[0]
                .fix
                .as_ref()
                .unwrap()
                .apply(source)
                .as_str(),
            r#"<div v-on:click="handleClick"></div>"#
        );
    }

    #[test]
    fn test_fix_preserves_modifiers() {
        let linter = create_linter(VOnStyleOption::Shorthand);
        let source = r#"<div v-on:click.stop.prevent="handleClick"></div>"#;
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert_eq!(
            result.diagnostics[0]
                .fix
                .as_ref()
                .unwrap()
                .apply(source)
                .as_str(),
            r#"<div @click.stop.prevent="handleClick"></div>"#
        );
    }
}
