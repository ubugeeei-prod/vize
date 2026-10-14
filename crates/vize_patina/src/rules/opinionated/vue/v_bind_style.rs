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

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::String;
use vize_relief::{DirectiveNode, ElementNode};

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
        // Skip object binding syntax (v-bind="...")
        if directive.name != "bind" || directive.arg.is_none() {
            return;
        }
        let start = directive.loc.span.start;
        let Some(text) = ctx
            .source
            .get(start as usize..directive.loc.span.end as usize)
        else {
            return;
        };

        // Only the prefix changes, so arguments, modifiers and the value keep
        // their authored text. `.prop` shorthand matches neither style.
        let (old, new, fix_message, message, help) = match self.style {
            VBindStyleOption::Shorthand if text.starts_with("v-bind:") => (
                "v-bind:",
                ":",
                "Use shorthand syntax",
                "Prefer shorthand `:` over `v-bind:`",
                "Use `:attr=\"value\"` instead of `v-bind:attr=\"value\"`",
            ),
            VBindStyleOption::Longform if text.starts_with(':') => (
                ":",
                "v-bind:",
                "Use longform syntax",
                "Prefer `v-bind:` over shorthand `:`",
                "Use `v-bind:attr=\"value\"` instead of `:attr=\"value\"`",
            ),
            _ => return,
        };
        let fix = Fix::new(
            fix_message,
            TextEdit::replace(start, start + old.len() as u32, String::from(new)),
        );
        ctx.report(
            LintDiagnostic::warn(META.name, message, start, directive.loc.span.end)
                .with_help(help)
                .with_fix(fix),
        );
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

    fn fixed(linter: &Linter, source: &str) -> std::string::String {
        let result = linter.lint_template(source, "test.vue");
        assert_eq!(result.warning_count, 1, "{source}");
        let edit = &result.diagnostics[0].fix.as_ref().unwrap().edits[0];
        let mut out = source.to_owned();
        out.replace_range(edit.start as usize..edit.end as usize, &edit.new_text);
        out
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

    #[test]
    fn fix_keeps_arguments_modifiers_and_values() {
        let shorthand = create_linter_shorthand();
        for (source, expected) in [
            (
                r#"<a v-bind:title.camel="t"></a>"#,
                r#"<a :title.camel="t"></a>"#,
            ),
            (
                r#"<a v-bind:[key]='x("y")'></a>"#,
                r#"<a :[key]='x("y")'></a>"#,
            ),
            (r#"<a v-bind:title></a>"#, r#"<a :title></a>"#),
        ] {
            assert_eq!(fixed(&shorthand, source), expected);
        }
        assert_eq!(
            fixed(&create_linter_longform(), r#"<a :title.prop="t"></a>"#),
            r#"<a v-bind:title.prop="t"></a>"#
        );
    }

    #[test]
    fn prop_shorthand_matches_neither_style() {
        for linter in [create_linter_shorthand(), create_linter_longform()] {
            let result = linter.lint_template(r#"<a .title="t"></a>"#, "test.vue");
            assert_eq!(result.warning_count, 0);
        }
    }
}
