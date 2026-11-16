//! ecosystem/void-link-require-href
//!
//! Require `href` on Void Vue `Link` components.
//!
//! Void's Vue adapter renders `Link` as the SPA navigation primitive. A missing
//! href leaves the component without a typed page target and fails at runtime.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::markup::{MarkupContext, MarkupElement, MarkupRule, MarkupTraversal};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

static META: RuleMeta = RuleMeta {
    name: "ecosystem/void-link-require-href",
    description: "Require `href` on Void Vue Link components",
    category: RuleCategory::Ecosystem,
    fixable: false,
    default_severity: Severity::Error,
};

/// Implements the `ecosystem/void-link-require-href` lint rule.
pub struct VoidLinkRequireHref;

impl MarkupRule for VoidLinkRequireHref {
    fn name(&self) -> &'static str {
        META.name
    }

    fn traversal(&self) -> MarkupTraversal {
        MarkupTraversal::Elements
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if !is_void_link_in_markup_context(ctx, element.tag())
            || has_named_markup_prop(element, "href")
        {
            return;
        }

        ctx.lint().error_at_with_help(
            "Void Vue Link components must declare an `href` target",
            element.range(),
            "Add `href` or `:href` so Void can type-check the page route and render the correct navigation element.",
        );
    }
}

impl Rule for VoidLinkRequireHref {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if !is_void_link_in_context(ctx, element.tag) || has_named_prop(element, "href") {
            return;
        }

        ctx.error_with_help(
            "Void Vue Link components must declare an `href` target",
            &element.loc,
            "Add `href` or `:href` so Void can type-check the page route and render the correct navigation element.",
        );
    }
}

pub(super) fn is_void_link(source: &str, tag: &str) -> bool {
    tag == "Link" && imports_void_vue(source)
}

pub(super) fn is_void_link_in_context(ctx: &LintContext<'_>, tag: &str) -> bool {
    let source = ctx
        .sfc_descriptor()
        .map(|descriptor| descriptor.source.as_ref())
        .unwrap_or(ctx.source);
    is_void_link(source, tag)
}

pub(super) fn is_void_link_in_markup_context(ctx: &mut MarkupContext<'_, '_>, tag: &str) -> bool {
    let lint = ctx.lint();
    let source = lint
        .sfc_descriptor()
        .map(|descriptor| descriptor.source.as_ref())
        .unwrap_or(lint.source);
    is_void_link(source, tag)
}

pub(super) fn has_named_markup_prop(element: &MarkupElement<'_>, name: &str) -> bool {
    let mut found = false;
    element.walk_attributes(&mut |attribute| {
        found |= attribute.name_eq(name);
    });
    element.walk_directives(&mut |directive| {
        found |= directive.name_eq("bind") && directive.is_static_arg_exact(name);
    });
    found
}

pub(super) fn static_markup_attr_value<'a>(
    element: &MarkupElement<'a>,
    name: &str,
) -> Option<&'a str> {
    let mut value = None;
    element.walk_attributes(&mut |attribute| {
        if attribute.name_eq(name) {
            value = attribute.value();
        }
    });
    value
}

pub(super) fn has_named_prop(element: &ElementNode<'_>, name: &str) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attr) => attr.name == name,
        PropNode::Directive(directive) => is_named_bind_directive(directive, name),
    })
}

pub(super) fn static_attr_value<'a>(element: &'a ElementNode<'a>, name: &str) -> Option<&'a str> {
    element.props.iter().find_map(|prop| match prop {
        PropNode::Attribute(attr) if attr.name == name => {
            attr.value.as_ref().map(|value| value.content)
        }
        _ => None,
    })
}

fn is_named_bind_directive(directive: &DirectiveNode<'_>, name: &str) -> bool {
    if directive.name != "bind" {
        return false;
    }

    matches!(
        directive.arg.as_ref(),
        Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == name
    )
}

fn imports_void_vue(source: &str) -> bool {
    source.contains("from \"@void/vue\"")
        || source.contains("from '@void/vue'")
        || source.contains("from \"@void/vue/client\"")
        || source.contains("from '@void/vue/client'")
}

#[cfg(test)]
mod tests {
    use super::VoidLinkRequireHref;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VoidLinkRequireHref));
        Linter::with_registry(registry)
    }

    #[test]
    fn accepts_void_link_href() {
        let source = r#"<script setup>
import { Link } from "@void/vue";
</script>
<template><Link href="/settings">Settings</Link></template>"#;

        let result = create_linter().lint_sfc(source, "test.vue");
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn reports_missing_void_link_href() {
        let source = r#"<script setup>
import { Link } from "@void/vue";
</script>
<template><Link>Settings</Link></template>"#;

        let result = create_linter().lint_sfc(source, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn reports_dynamic_void_link_href_argument() {
        let source = r#"<script setup>
import { Link } from "@void/vue";
</script>
<template><Link :[href]="url">Settings</Link></template>"#;

        let result = create_linter().lint_sfc(source, "test.vue");
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn ignores_non_void_link_components() {
        let result = create_linter().lint_template(r#"<Link>Plain link</Link>"#, "test.vue");
        assert_eq!(result.error_count, 0);
    }
}
