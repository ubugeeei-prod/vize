//! vue/no-boolean-attr-value
//!
//! Warn when boolean HTML attributes have explicit values.
//! For example, `disabled="disabled"` should be just `disabled`.
//! Based on markuplint's `no-boolean-attr-value` rule.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <template>
//!   <input disabled="disabled" />
//!   <input checked="checked" />
//!   <button disabled="true">Click</button>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <template>
//!   <input disabled />
//!   <input checked />
//!   <button disabled>Click</button>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::ir::ByteRange;
use crate::markup::{MarkupBinding, MarkupBindingKind, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use crate::rules::html::helpers::BOOLEAN_ATTRIBUTES;
use vize_l0::is_native_tag;
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-boolean-attr-value",
    description: "Disallow explicit values for boolean HTML attributes",
    category: RuleCategory::Recommended,
    fixable: true,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct NoBooleanAttrValue;

impl NoBooleanAttrValue {
    fn check_binding(ctx: &mut LintContext<'_>, binding: &MarkupBinding<'_>) {
        if binding.kind() != MarkupBindingKind::Attribute {
            return;
        }
        let Some(value) = binding.static_value() else {
            return;
        };

        let Some(name) = BOOLEAN_ATTRIBUTES
            .iter()
            .copied()
            .find(|name| binding.is_static_unqualified_arg_exact(name))
        else {
            return;
        };

        let message = ctx.t_fmt(
            "vue/no-boolean-attr-value.message",
            &[("attr", name), ("value", value)],
        );
        let help = ctx.t_fmt("vue/no-boolean-attr-value.help", &[("attr", name)]);
        let range = binding.range();
        let mut diagnostic =
            LintDiagnostic::warn(ctx.current_rule, message, range.start, range.end);
        if let Some(processed) = ctx.help_level().process(help.as_str()) {
            diagnostic = diagnostic.with_help(processed);
        }
        if let Some(fix) = value_fix(ctx.source, range, name, value, help.as_str()) {
            diagnostic = diagnostic.with_fix(fix);
        }
        ctx.report(diagnostic);
    }

    fn check_element(ctx: &mut LintContext<'_>, element: &MarkupElement<'_>) {
        let tag = element.tag();
        if element.is_component() || !element.is_unqualified_tag_exact(tag) || !is_native_tag(tag) {
            return;
        }

        element.walk_bindings(&mut |binding| Self::check_binding(ctx, &binding));
    }
}

/// Keep the authored name and surrounding bytes; remove only its assignment.
fn value_fix(source: &str, range: ByteRange, name: &str, value: &str, help: &str) -> Option<Fix> {
    // `hidden` is enumerated: dropping `until-found` would disable discovery.
    if name == "hidden" && value.eq_ignore_ascii_case("until-found") {
        return None;
    }
    let attribute = source.get(range.start as usize..range.end as usize)?;
    let assignment = attribute.strip_prefix(name)?;
    if source
        .as_bytes()
        .get(range.end as usize)
        .is_some_and(|next| !matches!(*next, b' ' | b'\t' | b'\r' | b'\n' | 12 | b'/' | b'>'))
    {
        return None;
    }
    if !assignment
        .trim_start_matches([' ', '\t', '\r', '\n', '\u{c}'])
        .starts_with('=')
    {
        return None;
    }
    Some(Fix::new(
        help,
        TextEdit::delete(range.start + name.len() as u32, range.end),
    ))
}

impl MarkupRule for NoBooleanAttrValue {
    fn name(&self) -> &'static str {
        META.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        Self::check_element(ctx.lint(), element);
    }
}

impl Rule for NoBooleanAttrValue {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn jsx_needs_lowering(&self) -> bool {
        // The legacy JSX fallback descends into expression containers before
        // projecting nested elements. Keep this rule on the lowered markup path
        // until the direct JSX document visitor has equivalent coverage.
        true
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Self::check_element(ctx, &MarkupElement::new(element));
    }
}

#[cfg(test)]
mod tests {
    use super::{NoBooleanAttrValue, value_fix};
    use crate::ir::ByteRange;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoBooleanAttrValue));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_no_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input disabled />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_checked_no_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input checked />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_non_boolean_with_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input type="text" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_dynamic_binding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input :disabled="isDisabled" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_component_like_custom_tag() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<my-button disabled="disabled" /><MyButton hidden="hidden" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_disabled_with_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input disabled="disabled" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_checked_with_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<input checked="checked" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_disabled_true() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<button disabled="true">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_multiple() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<input disabled="disabled" required="required" />"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn test_invalid_hidden_with_value() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div hidden="hidden">text</div>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn fix_refuses_missing_separator_and_unproven_source_ranges() {
        let source = "disabled=\"disabled\"title=\"keep\"";
        let end = "disabled=\"disabled\"".len() as u32;
        assert!(
            value_fix(
                source,
                ByteRange::new(0, end),
                "disabled",
                "disabled",
                "fix"
            )
            .is_none()
        );
        assert!(
            value_fix(
                source,
                ByteRange::new(1, end),
                "disabled",
                "disabled",
                "fix"
            )
            .is_none()
        );
        assert!(
            value_fix(
                source,
                ByteRange::new(0, 200),
                "disabled",
                "disabled",
                "fix"
            )
            .is_none()
        );
    }
}
