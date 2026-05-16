//! ecosystem/router-link-require-to
//!
//! Require `to` on RouterLink-like components.
//!
//! Router links without an explicit target are inert in the runtime and cannot
//! participate in typed route completion. This rule covers both Vue Router and
//! Nuxt link components while leaving plain anchors to other HTML rules.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "ecosystem/router-link-require-to",
    description: "Require a `to` target on RouterLink and NuxtLink components",
    category: RuleCategory::Ecosystem,
    fixable: false,
    default_severity: Severity::Error,
};

pub struct RouterLinkRequireTo;

impl Rule for RouterLinkRequireTo {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if !is_router_link_tag(element.tag.as_str()) || has_to_prop(element) {
            return;
        }

        ctx.error_with_help(
            "RouterLink components must declare a `to` target",
            &element.loc,
            "Add `to` or `:to` so navigation is explicit and typed-route tooling can infer the target.",
        );
    }
}

pub(super) fn is_router_link_tag(tag: &str) -> bool {
    matches!(tag, "RouterLink" | "router-link" | "NuxtLink" | "nuxt-link")
}

fn has_to_prop(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) => attr.name.as_str() == "to",
        PropNode::Directive(directive) => is_to_bind_directive(directive),
    })
}

fn is_to_bind_directive(directive: &DirectiveNode<'_>) -> bool {
    if directive.name.as_str() != "bind" {
        return false;
    }

    matches!(
        directive.arg.as_ref(),
        Some(ExpressionNode::Simple(arg)) if arg.content.as_str() == "to"
    )
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests {
    use super::RouterLinkRequireTo;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(RouterLinkRequireTo));
        Linter::with_registry(registry)
    }

    #[test]
    fn accepts_static_to() {
        let result = create_linter().lint_template(r#"<RouterLink to="/docs" />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn accepts_bound_to() {
        let result =
            create_linter().lint_template(r#"<router-link :to="{ name: 'home' }" />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn reports_missing_to() {
        let result = create_linter().lint_template(r#"<NuxtLink>Home</NuxtLink>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }
}
