//! vapor/no-suspense
//!
//! Warn about Suspense usage in Vapor-only applications.
//!
//! Suspense is NOT supported in Vapor-only mode. It only works when Vapor
//! components render inside a VDOM Suspense boundary.
//!
//! Based on Vue 3.6.0-beta.1 release notes:
//! <https://github.com/vuejs/core/releases/tag/v3.6.0-beta.1>
//!
//! ## Examples
//!
//! ### Invalid (in Vapor-only app)
//! ```vue
//! <Suspense>
//!   <AsyncComponent />
//! </Suspense>
//! ```
//!
//! ### Valid (in VDOM app with vaporInteropPlugin)
//! Suspense works when Vapor components are nested inside VDOM Suspense.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "vapor/no-suspense",
    description: "Warn about Suspense in Vapor-only apps (not supported)",
    category: RuleCategory::Vapor,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Warn about Suspense usage in Vapor-only apps
pub struct NoSuspense;

impl Rule for NoSuspense {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag.as_str() == "Suspense" || element.tag.as_str() == "suspense" {
            ctx.warn_with_help(
                ctx.t("vapor/no-suspense.message"),
                &element.loc,
                ctx.t("vapor/no-suspense.help"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoSuspense;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NoSuspense));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_suspense_warning() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<Suspense><AsyncComponent /></Suspense>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        assert!(result.diagnostics[0].message.contains("Suspense"));
    }

    #[test]
    fn test_lowercase_suspense() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<suspense><AsyncComponent /></suspense>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_component() {
        let linter = create_linter();
        let result = linter.lint_template(r#"<div><MyComponent /></div>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
