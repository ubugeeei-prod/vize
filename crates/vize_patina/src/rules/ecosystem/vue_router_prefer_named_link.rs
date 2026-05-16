//! ecosystem/vue-router-prefer-named-link
//!
//! Prefer named route objects for static RouterLink targets.
//!
//! Vue Router typed routes can autocomplete both paths and names, but route
//! names preserve params and future route refactors much better than string
//! paths. This rule keeps template links aligned with typed editor assistance.

use super::router_link_require_to::is_router_link_tag;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "ecosystem/vue-router-prefer-named-link",
    description: "Prefer named route objects over static path strings in RouterLink",
    category: RuleCategory::Ecosystem,
    fixable: false,
    default_severity: Severity::Warning,
};

pub struct VueRouterPreferNamedLink;

impl Rule for VueRouterPreferNamedLink {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if !is_router_link_tag(element.tag.as_str()) {
            return;
        }

        for prop in &element.props {
            let PropNode::Attribute(attr) = prop else {
                continue;
            };
            if attr.name.as_str() != "to" {
                continue;
            }
            let Some(value) = &attr.value else {
                continue;
            };
            if !is_internal_path(value.content.as_str()) {
                continue;
            }

            ctx.warn_with_help(
                "Prefer a named route object for RouterLink targets",
                &attr.loc,
                "Use `:to=\"{ name: 'route-name' }\"` so route params and Vue Router 5 typed routes stay visible to the editor.",
            );
        }
    }
}

fn is_internal_path(value: &str) -> bool {
    value.starts_with('/') && !value.starts_with("//")
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests {
    use super::VueRouterPreferNamedLink;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VueRouterPreferNamedLink));
        Linter::with_registry(registry)
    }

    #[test]
    fn accepts_named_route_object() {
        let result =
            create_linter().lint_template(r#"<RouterLink :to="{ name: 'home' }" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_static_path() {
        let result = create_linter().lint_template(r#"<RouterLink to="/users" />"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn ignores_external_like_protocol_relative_path() {
        let result =
            create_linter().lint_template(r#"<RouterLink to="//cdn.test/app" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
