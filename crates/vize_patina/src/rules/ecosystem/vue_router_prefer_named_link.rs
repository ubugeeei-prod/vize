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
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode};

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
            if !is_static_path_to_prop(prop) {
                continue;
            }

            ctx.warn_with_help(
                "Prefer a named route object for RouterLink targets",
                prop.loc(),
                "Use `:to=\"{ name: 'route-name' }\"` so route params and Vue Router 5 typed routes stay visible to the editor.",
            );
        }
    }
}

fn is_static_path_to_prop(prop: &PropNode<'_>) -> bool {
    match prop {
        PropNode::Attribute(attr) => {
            attr.name.as_str() == "to"
                && attr
                    .value
                    .as_ref()
                    .is_some_and(|value| is_internal_path(value.content.as_str()))
        }
        PropNode::Directive(directive) => is_static_path_to_bind(directive),
    }
}

fn is_static_path_to_bind(directive: &DirectiveNode<'_>) -> bool {
    if directive.name.as_str() != "bind" {
        return false;
    }
    if !matches!(
        directive.arg.as_ref(),
        Some(ExpressionNode::Simple(arg)) if arg.content.as_str() == "to"
    ) {
        return false;
    }

    let Some(ExpressionNode::Simple(exp)) = directive.exp.as_ref() else {
        return false;
    };
    let exp = exp.content.trim();
    static_string_literal_value(exp).is_some_and(is_internal_path)
        || object_path_literal_value(exp).is_some_and(is_internal_path)
}

fn is_internal_path(value: &str) -> bool {
    value.starts_with('/') && !value.starts_with("//")
}

fn static_string_literal_value(value: &str) -> Option<&str> {
    let bytes = value.as_bytes();
    let quote = *bytes.first()?;
    if bytes.len() < 2 || (quote != b'\'' && quote != b'"') || bytes.last() != Some(&quote) {
        return None;
    }
    let inner = &value[1..value.len() - 1];
    (!inner.contains('\\')).then_some(inner)
}

fn object_path_literal_value(value: &str) -> Option<&str> {
    let path_pos = value.find("path")?;
    let after_path = &value[path_pos + "path".len()..];
    let after_colon = after_path.trim_start().strip_prefix(':')?.trim_start();
    static_string_literal_value(after_colon.split([',', '}']).next()?.trim())
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
    fn reports_bound_static_string_and_object_path() {
        let result = create_linter().lint_template(
            r#"<div><RouterLink :to="'/users'" /><RouterLink :to="{ path: '/settings' }" /></div>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn ignores_external_like_protocol_relative_path() {
        let result =
            create_linter().lint_template(r#"<RouterLink to="//cdn.test/app" />"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }
}
