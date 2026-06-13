//! vue/no-multiple-objects-in-class
//!
//! Disallow multiple object literals inside a `:class` array binding.
//!
//! `:class="[{ a }, { b }]"` spreads class state across several object
//! literals when a single merged object reads more clearly and produces the
//! same result. Prefer `:class="{ a, b }"`.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div :class="[{ a }, { b }]"></div>
//! <div :class="[{ active: isActive }, { error: hasError }]"></div>
//! ```
//!
//! ### Valid
//! ```vue
//! <div :class="{ a, b }"></div>
//! <div :class="[{ active: isActive }, 'static']"></div>
//! <div :class="[foo, bar]"></div>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode};

static META: RuleMeta = RuleMeta {
    name: "vue/no-multiple-objects-in-class",
    description: "Disallow multiple object literals inside a :class array binding",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow multiple object literals inside a `:class` array binding.
pub struct NoMultipleObjectsInClass;

impl Rule for NoMultipleObjectsInClass {
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
        // Only a static `:class` argument, not `v-bind="obj"`.
        let Some(ExpressionNode::Simple(arg)) = &directive.arg else {
            return;
        };
        if arg.content.as_str() != "class" {
            return;
        }
        let Some(ExpressionNode::Simple(exp)) = &directive.exp else {
            return;
        };
        if count_top_level_objects_in_array(exp.content.as_str()) >= 2 {
            ctx.warn_with_help(
                ctx.t("vue/no-multiple-objects-in-class.message"),
                &directive.loc,
                ctx.t("vue/no-multiple-objects-in-class.help"),
            );
        }
    }
}

/// Count the object literals (`{ ... }` groups) that sit directly at the top
/// level of an array-literal expression string.
///
/// Returns `0` when the expression is not an array literal (does not start with
/// `[` and end with `]`). Nested braces/brackets are skipped so only the
/// array's own elements are counted; this is a pragmatic scan rather than a
/// full parse, which is sufficient for the heuristic.
fn count_top_level_objects_in_array(raw: &str) -> usize {
    let s = raw.trim();
    let bytes = s.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'[' || bytes[bytes.len() - 1] != b']' {
        return 0;
    }

    let mut count = 0usize;
    // Depth relative to the array body: `[` opens at depth 1, the array's own
    // elements live at depth 1, anything deeper is nested.
    let mut depth = 0i32;
    for &b in bytes {
        match b {
            b'[' => depth += 1,
            b']' => depth -= 1,
            // A `{` that opens while we are directly inside the array body
            // (depth 1) starts a top-level object literal.
            b'{' => {
                if depth == 1 {
                    count += 1;
                }
                depth += 1;
            }
            b'}' => depth -= 1,
            _ => {}
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::NoMultipleObjectsInClass;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoMultipleObjectsInClass));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_two_object_literals() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="[{ a }, { b }]"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_two_objects_with_keys() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :class="[{ active: isActive }, { error: hasError }]"></div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn allows_single_object_in_array() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :class="[{ active: isActive }, 'static']"></div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_plain_object_binding() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="{ a, b }"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn allows_array_of_identifiers() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :class="[foo, bar]"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn ignores_nested_objects() {
        // A single top-level object whose value is an object must not be
        // miscounted as two.
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<div :class="[{ a: { nested: true } }]"></div>"#,
            "App.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn ignores_other_bindings() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div :style="[{ a }, { b }]"></div>"#, "App.vue");
        assert_eq!(result.warning_count, 0);
    }
}
