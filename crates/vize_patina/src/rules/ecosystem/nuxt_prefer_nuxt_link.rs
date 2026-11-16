//! ecosystem/nuxt-prefer-nuxt-link
//!
//! Prefer NuxtLink for internal navigation.
//!
//! NuxtLink integrates with Nuxt's router, prefetching, and typed route
//! experience. Plain anchors remain useful for external URLs, downloads, and
//! intentionally new-window navigation.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "ecosystem/nuxt-prefer-nuxt-link",
    description: "Prefer NuxtLink for internal application links",
    category: RuleCategory::Ecosystem,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Implements the `ecosystem/nuxt-prefer-nuxt-link` lint rule.
pub struct NuxtPreferNuxtLink;

impl MarkupRule for NuxtPreferNuxtLink {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if element.tag() != "a" || has_markup_anchor_escape_hatch(element) {
            return;
        }

        element.walk_attributes(&mut |attribute| {
            if attribute.name_eq("href")
                && attribute.value().is_some_and(is_internal_href)
            {
                ctx.lint().warn_at_with_help(
                    "Use NuxtLink for internal links",
                    attribute.range(),
                    "Replace the anchor with `<NuxtLink to=\"...\">` so Nuxt can prefetch, route, and type-check the navigation target.",
                );
            }
        });
    }
}

impl Rule for NuxtPreferNuxtLink {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if element.tag != "a" || has_anchor_escape_hatch(element) {
            return;
        }

        for prop in &element.props {
            let PropNode::Attribute(attr) = prop else {
                continue;
            };
            if attr.name != "href" {
                continue;
            }
            let Some(value) = &attr.value else {
                continue;
            };
            if !is_internal_href(value.content) {
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

fn has_markup_anchor_escape_hatch(element: &MarkupElement<'_>) -> bool {
    let mut found = false;
    element.walk_attributes(&mut |attribute| {
        found |= attribute.name_eq("download")
            || (attribute.name_eq("target") && attribute.value() == Some("_blank"));
    });
    found
}

fn has_anchor_escape_hatch(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) if attr.name == "download" => true,
        PropNode::Attribute(attr) if attr.name == "target" => attr
            .value
            .as_ref()
            .is_some_and(|value| value.content == "_blank"),
        _ => false,
    })
}

fn is_internal_href(value: &str) -> bool {
    value.starts_with('/') && !value.starts_with("//")
}

#[cfg(test)]
#[expect(
    clippy::disallowed_macros,
    reason = "snapshot fixture uses insta's format!"
)]
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
