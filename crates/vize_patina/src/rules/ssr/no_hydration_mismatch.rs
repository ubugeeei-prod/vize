//! Rule: no-hydration-mismatch
//!
//! Warns when non-deterministic values are used in templates that could cause
//! hydration mismatches in SSR.
//!
//! ## Why is this bad?
//! In SSR, the server renders HTML first, then the client hydrates it.
//! If the values differ between server and client, Vue will show a warning
//! and potentially re-render the component, negating SSR benefits.
//!
//! Common causes of hydration mismatch:
//! - Random values (Math.random(), crypto.randomUUID())
//! - Current time (Date.now(), new Date())
//! - Environment-specific values
//!
//! ## How to fix?
//! - Use `useId()` instead of random IDs
//! - Move non-deterministic logic to `onMounted` or use `<ClientOnly>`
//! - Use `useDateFormat` with a fixed date on server
//!
//! ## Example
//!
//! Bad:
//! ```vue
//! <template>
//!   <div :id="`item-${Math.random()}`">
//!     Current time: {{ Date.now() }}
//!   </div>
//! </template>
//! ```
//!
//! Good:
//! ```vue
//! <script setup>
//! import { useId } from 'vue';
//! const id = useId();
//! </script>
//!
//! <template>
//!   <div :id="`item-${id}`">
//!     <!-- Time shown only on client -->
//!     <ClientOnly>
//!       Current time: {{ Date.now() }}
//!     </ClientOnly>
//!   </div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::{ChainElement, Expression};
use oxc_ast_visit::{Visit, walk::walk_expression};
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use vize_relief::ast::{ElementNode, ExpressionNode, InterpolationNode};

/// Non-deterministic function/value patterns that cause hydration mismatch
const HYDRATION_MISMATCH_PATTERNS: &[(&str, &str)] = &[
    // Random values
    (
        "Math.random",
        "Random values differ between server and client. Use `useId()` for unique IDs",
    ),
    (
        "crypto.randomUUID",
        "Random UUIDs differ between server and client. Use `useId()` for unique IDs",
    ),
    (
        "crypto.getRandomValues",
        "Random values differ between server and client",
    ),
    (
        "Math.floor(Math.random",
        "Random values differ between server and client. Use `useId()` for unique IDs",
    ),
    (
        "uuid()",
        "Random UUIDs differ between server and client. Use `useId()` for unique IDs",
    ),
    (
        "nanoid()",
        "Random IDs differ between server and client. Use `useId()` for unique IDs",
    ),
    // Date/Time
    (
        "Date.now",
        "Current time differs between server and client. Consider using `<ClientOnly>` or a fixed timestamp",
    ),
    (
        "new Date()",
        "Current time differs between server and client. Consider using `<ClientOnly>` or a fixed timestamp",
    ),
    (
        ".getTime()",
        "Time values may differ between server and client",
    ),
    (
        ".toLocaleString()",
        "Locale formatting may differ between server and client environments",
    ),
    (
        ".toLocaleDateString()",
        "Locale formatting may differ between server and client environments",
    ),
    (
        ".toLocaleTimeString()",
        "Locale formatting may differ between server and client environments",
    ),
    // Performance timing
    (
        "performance.now",
        "Performance timing differs between server and client",
    ),
    // Environment-specific
    (
        "process.env",
        "Environment variables may differ between server and client. Ensure they are consistent or use runtime config",
    ),
    (
        "import.meta.env",
        "Environment variables may differ between server and client. Ensure they are consistent or use runtime config",
    ),
];

static META: RuleMeta = RuleMeta {
    name: "ssr/no-hydration-mismatch",
    description: "Disallow non-deterministic values that cause hydration mismatch",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

pub struct NoHydrationMismatch;

impl NoHydrationMismatch {
    /// Check if expression contains any mismatch-prone patterns
    fn check_expression(content: &str) -> Option<(&'static str, &'static str)> {
        let allocator = OxcAllocator::default();
        let source_type = SourceType::from_path("template.ts").unwrap_or_default();
        let Ok(expression) = OxcParser::new(&allocator, content, source_type).parse_expression()
        else {
            return None;
        };

        let mut visitor = HydrationMismatchVisitor::default();
        visitor.visit_expression(&expression);
        visitor.found
    }
}

#[derive(Default)]
struct HydrationMismatchVisitor {
    found: Option<(&'static str, &'static str)>,
}

impl<'a> Visit<'a> for HydrationMismatchVisitor {
    fn visit_expression(&mut self, expression: &Expression<'a>) {
        if self.found.is_none() {
            self.found = detect_hydration_mismatch(expression);
        }
        if self.found.is_some() {
            return;
        }
        walk_expression(self, expression);
    }
}

fn detect_hydration_mismatch(expression: &Expression<'_>) -> Option<(&'static str, &'static str)> {
    if matches_static_member_chain(expression, &["Math", "random"]) {
        return pattern_match("Math.random");
    }
    if matches_static_member_chain(expression, &["crypto", "randomUUID"]) {
        return pattern_match("crypto.randomUUID");
    }
    if matches_static_member_chain(expression, &["crypto", "getRandomValues"]) {
        return pattern_match("crypto.getRandomValues");
    }
    if matches_static_member_chain(expression, &["Date", "now"]) {
        return pattern_match("Date.now");
    }
    if matches_static_member_chain(expression, &["performance", "now"]) {
        return pattern_match("performance.now");
    }
    if matches_static_member_chain(expression, &["process", "env"]) {
        return pattern_match("process.env");
    }
    if is_import_meta_env(expression) {
        return pattern_match("import.meta.env");
    }

    match unwrap_expression(expression) {
        Expression::CallExpression(call) => {
            if let Expression::Identifier(identifier) = unwrap_expression(&call.callee) {
                match identifier.name.as_str() {
                    "uuid" => return pattern_match("uuid()"),
                    "nanoid" => return pattern_match("nanoid()"),
                    _ => {}
                }
            }

            let member = unwrap_expression(&call.callee).as_member_expression()?;

            match member.static_property_name() {
                Some("getTime") => pattern_match(".getTime()"),
                Some("toLocaleString") => pattern_match(".toLocaleString()"),
                Some("toLocaleDateString") => pattern_match(".toLocaleDateString()"),
                Some("toLocaleTimeString") => pattern_match(".toLocaleTimeString()"),
                _ => None,
            }
        }
        Expression::NewExpression(new_expression) => {
            if new_expression.arguments.is_empty()
                && let Expression::Identifier(identifier) =
                    unwrap_expression(&new_expression.callee)
                && identifier.name.as_str() == "Date"
            {
                return pattern_match("new Date()");
            }
            None
        }
        _ => None,
    }
}

fn unwrap_expression<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(paren) => unwrap_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => unwrap_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            unwrap_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => unwrap_expression(&ts_non_null.expression),
        _ => expression,
    }
}

fn matches_static_member_chain(expression: &Expression<'_>, expected: &[&str]) -> bool {
    let mut parts = Vec::with_capacity(expected.len());
    if !collect_static_member_chain(unwrap_expression(expression), &mut parts) {
        return false;
    }
    parts == expected
}

fn collect_static_member_chain<'a>(
    expression: &'a Expression<'a>,
    parts: &mut Vec<&'a str>,
) -> bool {
    match unwrap_expression(expression) {
        Expression::Identifier(identifier) => {
            parts.push(identifier.name.as_str());
            true
        }
        member if member.is_member_expression() => {
            let Some(member) = member.as_member_expression() else {
                return false;
            };
            if !collect_static_member_chain(member.object(), parts) {
                return false;
            }
            let Some(property) = member.static_property_name() else {
                return false;
            };
            parts.push(property);
            true
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::StaticMemberExpression(member) => {
                if !collect_static_member_chain(&member.object, parts) {
                    return false;
                }
                parts.push(member.property.name.as_str());
                true
            }
            ChainElement::TSNonNullExpression(non_null) => {
                collect_static_member_chain(&non_null.expression, parts)
            }
            _ => false,
        },
        Expression::MetaProperty(meta) => {
            parts.push(meta.meta.name.as_str());
            parts.push(meta.property.name.as_str());
            true
        }
        _ => false,
    }
}

fn is_import_meta_env(expression: &Expression<'_>) -> bool {
    matches_static_member_chain(expression, &["import", "meta", "env"])
}

fn pattern_match(pattern: &'static str) -> Option<(&'static str, &'static str)> {
    HYDRATION_MISMATCH_PATTERNS
        .iter()
        .find(|(candidate, _)| *candidate == pattern)
        .copied()
}

impl Rule for NoHydrationMismatch {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn check_interpolation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        interpolation: &InterpolationNode<'a>,
    ) {
        // Only run if SSR mode is enabled
        if !ctx.is_ssr_enabled() {
            return;
        }

        let content = match &interpolation.content {
            ExpressionNode::Simple(s) => s.content.as_str(),
            ExpressionNode::Compound(_) => return, // Skip compound expressions
        };

        if let Some((pattern, _help)) = Self::check_expression(content) {
            ctx.warn_with_help(
                ctx.t_fmt("ssr/no-hydration-mismatch.message", &[("pattern", pattern)]),
                &interpolation.loc,
                ctx.t_fmt("ssr/no-hydration-mismatch.help", &[("pattern", pattern)]),
            );
        }
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _element: &ElementNode<'a>,
        directive: &vize_relief::ast::DirectiveNode<'a>,
    ) {
        // Only run if SSR mode is enabled
        if !ctx.is_ssr_enabled() {
            return;
        }

        // Check directive expressions
        if let Some(exp) = &directive.exp {
            let content = match exp {
                ExpressionNode::Simple(s) => s.content.as_str(),
                ExpressionNode::Compound(_) => return, // Skip compound expressions
            };

            if let Some((pattern, _help)) = Self::check_expression(content) {
                ctx.warn_with_help(
                    ctx.t_fmt(
                        "ssr/no-hydration-mismatch.message-attr",
                        &[("pattern", pattern)],
                    ),
                    &directive.loc,
                    ctx.t_fmt("ssr/no-hydration-mismatch.help", &[("pattern", pattern)]),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoHydrationMismatch;

    #[test]
    fn test_detects_math_random() {
        let content = "items.map(() => Math.random())";
        assert!(NoHydrationMismatch::check_expression(content).is_some());
    }

    #[test]
    fn test_detects_date_now() {
        let content = "Date.now()";
        assert!(NoHydrationMismatch::check_expression(content).is_some());
    }

    #[test]
    fn test_detects_new_date() {
        let content = "new Date()";
        assert!(NoHydrationMismatch::check_expression(content).is_some());
    }

    #[test]
    fn test_detects_crypto_random() {
        let content = "crypto.randomUUID()";
        assert!(NoHydrationMismatch::check_expression(content).is_some());
    }

    #[test]
    fn test_allows_safe_code() {
        let content = "items.map(item => item.name)";
        assert!(NoHydrationMismatch::check_expression(content).is_none());
    }

    #[test]
    fn test_ignores_string_literal_false_positive() {
        let content = r#"'Date.now and Math.random are just text'"#;
        assert!(NoHydrationMismatch::check_expression(content).is_none());
    }

    #[test]
    fn test_ignores_template_literal_raw_text_false_positive() {
        let content = r#"`Date.now and import.meta.env are just text`"#;
        assert!(NoHydrationMismatch::check_expression(content).is_none());
    }

    #[test]
    fn test_detects_template_literal_embedded_expression() {
        let content = r#"`item-${Date.now()}`"#;
        assert_eq!(
            NoHydrationMismatch::check_expression(content).map(|(pattern, _)| pattern),
            Some("Date.now")
        );
    }

    #[test]
    fn test_ignores_unrelated_member_chain() {
        let content = "globals.Date.now";
        assert!(NoHydrationMismatch::check_expression(content).is_none());
    }

    #[test]
    fn test_detects_import_meta_env() {
        let content = "import.meta.env.MODE";
        assert_eq!(
            NoHydrationMismatch::check_expression(content).map(|(pattern, _)| pattern),
            Some("import.meta.env")
        );
    }
}
