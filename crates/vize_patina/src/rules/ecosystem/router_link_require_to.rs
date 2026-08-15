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
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

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
        let tag = element.tag;
        if !is_router_link_tag(tag)
            || has_navigation_target(element, is_nuxt_link_tag(tag))
            || can_inherit_root_navigation_target(ctx)
        {
            return;
        }

        ctx.error_with_help(
            "RouterLink components must declare a navigation target",
            &element.loc,
            "Add `to` or `:to` (`href` or `:href` is also valid on NuxtLink) so navigation is explicit and typed-route tooling can infer the target.",
        );
    }
}

pub(super) fn is_router_link_tag(tag: &str) -> bool {
    matches!(tag, "RouterLink" | "router-link" | "NuxtLink" | "nuxt-link")
}

fn is_nuxt_link_tag(tag: &str) -> bool {
    matches!(tag, "NuxtLink" | "nuxt-link")
}

fn can_inherit_root_navigation_target(ctx: &LintContext<'_>) -> bool {
    ctx.parent_element().is_none()
        && ctx.sfc_descriptor().is_some()
        && ctx.analysis().is_some_and(|analysis| {
            analysis.template_info.root_element_count <= 1
                && !analysis.macros.all_calls().iter().any(|call| {
                    call.name == "defineOptions"
                        && call.runtime_args.as_ref().is_some_and(|args| {
                            args.contains("inheritAttrs") && args.contains("false")
                        })
                })
        })
}

fn has_navigation_target(element: &ElementNode<'_>, allow_href: bool) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) => attr.name == "to" || (allow_href && attr.name == "href"),
        PropNode::Directive(directive) => is_navigation_bind_directive(directive, allow_href),
    })
}

fn is_navigation_bind_directive(directive: &DirectiveNode<'_>, allow_href: bool) -> bool {
    if directive.name != "bind" {
        return false;
    }

    match directive.arg.as_ref() {
        None => true,
        Some(ExpressionNode::Simple(arg)) if arg.is_static => {
            arg.content == "to" || (allow_href && arg.content == "href")
        }
        _ => false,
    }
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
    fn accepts_nuxt_href_aliases() {
        let linter = create_linter();
        let static_result = linter.lint_template(r#"<NuxtLink href="/docs" />"#, "test.vue");
        let bound_result = linter.lint_template(r#"<nuxt-link :href="route" />"#, "test.vue");

        assert_eq!(static_result.error_count, 0);
        assert_eq!(bound_result.error_count, 0);
    }

    #[test]
    fn accepts_object_v_bind_that_can_supply_the_target() {
        let result =
            create_linter().lint_template(r#"<RouterLink v-bind="linkProps" />"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn router_link_still_requires_to_instead_of_href() {
        let result = create_linter().lint_template(r#"<RouterLink href="/docs" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn accepts_single_root_link_that_inherits_fallthrough_attrs() {
        let result = create_linter().lint_sfc(
            r#"<script setup lang="ts">
defineProps<{ class?: string }>()
</script>
<template><NuxtLink class="card"><slot /></NuxtLink></template>"#,
            "LinkedCard.vue",
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn reports_single_root_link_when_attr_inheritance_is_disabled() {
        let result = create_linter().lint_sfc(
            r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false })
</script>
<template><NuxtLink class="card"><slot /></NuxtLink></template>"#,
            "LinkedCard.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_targetless_link_in_a_fragment() {
        let result = create_linter().lint_sfc(
            r#"<template><NuxtLink>Docs</NuxtLink><footer>Footer</footer></template>"#,
            "FragmentCard.vue",
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_dynamic_to_argument() {
        let result = create_linter().lint_template(r#"<router-link :[to]="route" />"#, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_missing_to() {
        let result = create_linter().lint_template(r#"<NuxtLink>Home</NuxtLink>"#, "test.vue");
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }
}
