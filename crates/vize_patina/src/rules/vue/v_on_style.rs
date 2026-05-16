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
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::String;
use vize_relief::ast::{DirectiveNode, ElementNode};

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
    Shorthand,
    Longform,
}

/// Enforce v-on directive style
pub struct VOnStyle {
    pub style: VOnStyleOption,
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

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        if directive.name.as_str() != "on" {
            return;
        }

        // Skip object binding syntax (v-on="...")
        if directive.arg.is_none() {
            return;
        }

        let raw_name = directive.raw_name.as_deref().unwrap_or("");
        let is_shorthand = raw_name.starts_with('@');

        match self.style {
            VOnStyleOption::Shorthand => {
                if !is_shorthand {
                    if let Some(new_text) =
                        replacement_text(ctx, directive, VOnStyleOption::Shorthand)
                    {
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
                                "Prefer shorthand `@` over `v-on:`",
                                directive.loc.start.offset,
                                directive.loc.end.offset,
                            )
                            .with_help(
                                "Use `@event=\"handler\"` instead of `v-on:event=\"handler\"`",
                            )
                            .with_fix(fix),
                        );
                    }
                }
            }
            VOnStyleOption::Longform => {
                if is_shorthand {
                    if let Some(new_text) =
                        replacement_text(ctx, directive, VOnStyleOption::Longform)
                    {
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
                                "Prefer `v-on:` over shorthand `@`",
                                directive.loc.start.offset,
                                directive.loc.end.offset,
                            )
                            .with_help(
                                "Use `v-on:event=\"handler\"` instead of `@event=\"handler\"`",
                            )
                            .with_fix(fix),
                        );
                    }
                }
            }
        }
    }
}

fn replacement_text<'a>(
    ctx: &LintContext<'a>,
    directive: &DirectiveNode<'a>,
    target: VOnStyleOption,
) -> Option<String> {
    let raw_name = directive.raw_name.as_deref()?;
    let start = directive.loc.start.offset as usize;
    let end = directive.loc.end.offset as usize;
    let suffix_start = start.checked_add(raw_name.len())?;
    let rest = ctx.source.get(suffix_start..end)?;

    match target {
        VOnStyleOption::Shorthand => {
            raw_name.strip_prefix("v-on")?;
            let rest = rest.strip_prefix(':')?;
            let mut new_text = String::with_capacity(1 + rest.len());
            new_text.push('@');
            new_text.push_str(rest);
            Some(new_text)
        }
        VOnStyleOption::Longform => {
            raw_name.strip_prefix('@')?;
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
