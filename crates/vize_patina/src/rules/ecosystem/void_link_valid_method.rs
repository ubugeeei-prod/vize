//! ecosystem/void-link-valid-method
//!
//! Validate static Void Vue `Link` method props.
//!
//! Void renders GET links as anchors and mutation links as buttons. Static
//! method typos or GET-only props on mutation links are easy to catch locally.

use super::void_link_require_href::{has_named_prop, is_void_link_in_context, static_attr_value};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ElementNode;

static META: RuleMeta = RuleMeta {
    name: "ecosystem/void-link-valid-method",
    description: "Validate static Void Vue Link method props",
    category: RuleCategory::Ecosystem,
    fixable: false,
    default_severity: Severity::Warning,
};

pub struct VoidLinkValidMethod;

impl Rule for VoidLinkValidMethod {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if !is_void_link_in_context(ctx, element.tag.as_str()) {
            return;
        }

        let Some(method) = static_attr_value(element, "method") else {
            return;
        };
        if !is_valid_method(method) {
            ctx.warn_with_help(
                "Void Vue Link uses an unknown navigation method",
                &element.loc,
                "Use one of `GET`, `POST`, `PUT`, `PATCH`, or `DELETE` for the static `method` prop.",
            );
            return;
        }

        if is_get(method) {
            return;
        }

        if has_named_prop(element, "prefetch") || has_named_prop(element, "reloadDocument") {
            ctx.warn_with_help(
                "Void Vue Link GET-only props are ignored on mutation links",
                &element.loc,
                "Remove `prefetch` and `reloadDocument`, or switch the Link method back to `GET`.",
            );
        }
    }
}

fn is_valid_method(method: &str) -> bool {
    matches!(
        method.to_ascii_uppercase().as_str(),
        "GET" | "POST" | "PUT" | "PATCH" | "DELETE"
    )
}

fn is_get(method: &str) -> bool {
    method.eq_ignore_ascii_case("GET")
}

#[cfg(test)]
mod tests {
    use super::VoidLinkValidMethod;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(VoidLinkValidMethod));
        Linter::with_registry(registry)
    }

    #[test]
    fn accepts_valid_static_method() {
        let source = r#"<script setup>
import { Link } from "@void/vue";
</script>
<template><Link href="/posts" method="POST" /></template>"#;

        let result = create_linter().lint_sfc(source, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_unknown_static_method() {
        let source = r#"<script setup>
import { Link } from "@void/vue";
</script>
<template><Link href="/posts" method="CREATE" /></template>"#;

        let result = create_linter().lint_sfc(source, "test.vue");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn reports_get_only_props_on_mutation_link() {
        let source = r#"<script setup>
import { Link } from "@void/vue";
</script>
<template><Link href="/posts" method="DELETE" prefetch /></template>"#;

        let result = create_linter().lint_sfc(source, "test.vue");
        assert_eq!(result.warning_count, 1);
    }
}
