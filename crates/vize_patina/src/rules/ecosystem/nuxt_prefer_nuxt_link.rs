//! ecosystem/nuxt-prefer-nuxt-link
//!
//! Prefer NuxtLink for internal navigation.
//!
//! NuxtLink integrates with Nuxt's router, prefetching, and typed route
//! experience. Plain anchors remain useful for external URLs, downloads, and
//! intentionally new-window navigation.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "ecosystem/nuxt-prefer-nuxt-link",
    description: "Prefer NuxtLink for internal application links",
    category: RuleCategory::Ecosystem,
    fixable: false,
    default_severity: Severity::Warning,
};

pub struct NuxtPreferNuxtLink;

impl Rule for NuxtPreferNuxtLink {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag.as_str() != "a" || has_anchor_escape_hatch(element) {
            return;
        }

        for prop in &element.props {
            let PropNode::Attribute(attr) = prop else {
                continue;
            };
            if attr.name.as_str() != "href" {
                continue;
            }
            let Some(value) = &attr.value else {
                continue;
            };
            if !is_internal_href(value.content.as_str()) {
                continue;
            }

            ctx.warn_with_help(
                "Use NuxtLink for internal links",
                &attr.loc,
                "Replace the anchor with `<NuxtLink to=\"...\">` so Nuxt can prefetch, route, and type-check the navigation target.",
            );
        }
    }
}

fn has_anchor_escape_hatch(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) if attr.name.as_str() == "download" => true,
        PropNode::Attribute(attr) if attr.name.as_str() == "target" => attr
            .value
            .as_ref()
            .is_some_and(|value| value.content.as_str() == "_blank"),
        _ => false,
    })
}

fn is_internal_href(value: &str) -> bool {
    value.starts_with('/') && !value.starts_with("//")
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests {
    use super::NuxtPreferNuxtLink;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(NuxtPreferNuxtLink));
        Linter::with_registry(registry)
    }

    #[test]
    fn reports_internal_anchor() {
        let result =
            create_linter().lint_template(r#"<a href="/settings">Settings</a>"#, "test.vue");
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn accepts_external_anchor() {
        let result = create_linter()
            .lint_template(r#"<a href="https://example.com">External</a>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn accepts_download_anchor() {
        let result =
            create_linter().lint_template(r#"<a href="/report.pdf" download>PDF</a>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
