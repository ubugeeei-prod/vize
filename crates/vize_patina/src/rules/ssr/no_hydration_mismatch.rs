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
use vize_relief::ast::{ElementNode, ExpressionNode, InterpolationNode};

/// Non-deterministic function/value patterns that cause hydration mismatch
const HYDRATION_MISMATCH_PATTERNS: &[(&str, &str)] = &[
    // Random values
    ("Math.random", "Random values differ between server and client. Use `useId()` for unique IDs"),
    ("crypto.randomUUID", "Random UUIDs differ between server and client. Use `useId()` for unique IDs"),
    ("crypto.getRandomValues", "Random values differ between server and client"),
    ("Math.floor(Math.random", "Random values differ between server and client. Use `useId()` for unique IDs"),
    ("uuid()", "Random UUIDs differ between server and client. Use `useId()` for unique IDs"),
    ("nanoid()", "Random IDs differ between server and client. Use `useId()` for unique IDs"),

    // Date/Time
    ("Date.now", "Current time differs between server and client. Consider using `<ClientOnly>` or a fixed timestamp"),
    ("new Date()", "Current time differs between server and client. Consider using `<ClientOnly>` or a fixed timestamp"),
    (".getTime()", "Time values may differ between server and client"),
    (".toLocaleString()", "Locale formatting may differ between server and client environments"),
    (".toLocaleDateString()", "Locale formatting may differ between server and client environments"),
    (".toLocaleTimeString()", "Locale formatting may differ between server and client environments"),

    // Performance timing
    ("performance.now", "Performance timing differs between server and client"),

    // Environment-specific
    ("process.env", "Environment variables may differ between server and client. Ensure they are consistent or use runtime config"),
    ("import.meta.env", "Environment variables may differ between server and client. Ensure they are consistent or use runtime config"),
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
        for (pattern, help) in HYDRATION_MISMATCH_PATTERNS {
            if content.contains(pattern) {
                return Some((pattern, help));
            }
        }
        None
    }
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
}
