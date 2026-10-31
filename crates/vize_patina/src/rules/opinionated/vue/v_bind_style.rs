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
use vize_l0::{String, cstr};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

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

        if self.style == VBindStyleOption::Longform && text.starts_with('.') {
            let mut diagnostic = LintDiagnostic::warn(
                META.name,
                "Expected 'v-bind:' instead of '.'.",
                start,
                directive.loc.span.end,
            )
            .with_help("Use `v-bind:attr.prop=\"value\"` instead of `.attr=\"value\"`");
            if let Some(fix) = property_shorthand_fix(ctx.source, directive) {
                diagnostic = diagnostic.with_fix(fix);
            }
            ctx.report(diagnostic);
            return;
        }

        // The ordinary forms only change the prefix. Property shorthand also
        // needs its implicit `.prop` semantics retained above.
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

fn property_shorthand_fix(source: &str, directive: &DirectiveNode<'_>) -> Option<Fix> {
    let start = directive.loc.span.start;
    let prefix_end = start.checked_add(1)?;
    // The finalized public parser retains the spelling prefix, not the full
    // authored head. Argument/modifier spans retain the remaining head bytes.
    if directive.raw_name != Some(".")
        || source.get(start as usize..prefix_end as usize) != Some(".")
    {
        return None;
    }
    // The parser also supplies a synthetic leading `prop` modifier for `.foo`.
    // Only a modifier whose exact authored bytes are `prop` is already written.
    let explicit_prop = directive.modifiers.iter().any(|modifier| {
        modifier.content == "prop"
            && modifier.loc.span.start >= prefix_end
            && modifier.loc.span.end <= directive.loc.span.end
            && source.get(modifier.loc.span.start as usize..modifier.loc.span.end as usize)
                == Some("prop")
    });
    let (end, replacement) = if explicit_prop {
        (start.checked_add(1)?, String::from("v-bind:"))
    } else {
        let ExpressionNode::Simple(argument) = directive.arg.as_ref()? else {
            return None;
        };
        let prefix = if argument.is_static { 1 } else { 2 };
        if argument.loc.span.start != start.checked_add(prefix)? {
            return None;
        }
        if !argument.is_static
            && (source.get(start as usize..start as usize + 2) != Some(".[")
                || source.as_bytes().get(argument.loc.span.end as usize) != Some(&b']'))
        {
            return None;
        }
        let end = argument
            .loc
            .span
            .end
            .checked_add(u32::from(!argument.is_static))?;
        if end > directive.loc.span.end {
            return None;
        }
        let argument = source.get(start.checked_add(1)? as usize..end as usize)?;
        (end, cstr!("v-bind:{argument}.prop"))
    };
    Some(Fix::new(
        "Use longform syntax",
        TextEdit::replace(start, end, replacement),
    ))
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
    fn prop_shorthand_matches_shorthand_and_requires_explicit_longform() {
        for (linter, warnings) in [
            (create_linter_shorthand(), 0),
            (create_linter_longform(), 1),
        ] {
            let result = linter.lint_template(r#"<a .title="t"></a>"#, "test.vue");
            assert_eq!(result.warning_count, warnings);
            if warnings == 1 {
                let [diagnostic] = result.diagnostics.as_slice() else {
                    panic!("one complete property-shorthand diagnostic")
                };
                assert_eq!(
                    diagnostic
                        .fix
                        .as_ref()
                        .unwrap()
                        .apply(r#"<a .title="t"></a>"#),
                    r#"<a v-bind:title.prop="t"></a>"#
                );
            }
        }
    }
}
