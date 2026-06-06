//! ecosystem/vue-router-prefer-named-push
//!
//! Prefer named route objects for programmatic navigation.
//!
//! Vue Router 5 brings file-based typed routes into the core package. Named
//! navigation keeps params explicit and gives editors a compact place to surface
//! route-name completions.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use memchr::memmem;
use oxc_ast::ast::{
    Argument, CallExpression, Expression, ObjectExpression, ObjectPropertyKind, Program,
    PropertyKey,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::Span;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "ecosystem/vue-router-prefer-named-push",
    description: "Prefer named route objects for Vue Router programmatic navigation",
    default_severity: Severity::Warning,
};

pub struct VueRouterPreferNamedPush;

impl ScriptRule for VueRouterPreferNamedPush {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let bytes = source.as_bytes();
        if (memmem::find(bytes, b".push").is_none() && memmem::find(bytes, b".replace").is_none())
            || (memmem::find(bytes, b"'/").is_none() && memmem::find(bytes, b"\"/").is_none())
            || (memmem::find(bytes, b"router").is_none()
                && memmem::find(bytes, b"Router").is_none())
        {
            return;
        }

        let mut visitor = RouterPushVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct RouterPushVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for RouterPushVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if is_router_navigation_call(it)
            && let Some(span) = static_path_argument_span(it.arguments.first())
        {
            self.result.add_diagnostic(
                    LintDiagnostic::warn(
                        META.name,
                        "Prefer a named route object for router navigation",
                        self.offset as u32 + span.start,
                        self.offset as u32 + span.end,
                    )
                    .with_help(
                        "Use `router.push({ name: 'route-name' })` or `router.replace({ name: 'route-name' })` so typed routes can validate params and editor completions.",
                    ),
                );
        }

        walk_call_expression(self, it);
    }
}

fn is_router_navigation_call(call: &CallExpression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };

    matches!(member.property.name.as_str(), "push" | "replace")
}

fn static_path_argument_span(argument: Option<&Argument<'_>>) -> Option<Span> {
    match argument? {
        Argument::StringLiteral(literal) if is_internal_path(literal.value.as_str()) => {
            Some(literal.span)
        }
        Argument::ObjectExpression(object) => object_path_literal_span(object),
        _ => None,
    }
}

fn object_path_literal_span(object: &ObjectExpression<'_>) -> Option<Span> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property.computed || property_key_name(&property.key) != Some("path") {
            continue;
        }
        let Expression::StringLiteral(literal) = &property.value else {
            continue;
        };
        if is_internal_path(literal.value.as_str()) {
            return Some(literal.span);
        }
    }
    None
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

fn is_internal_path(value: &str) -> bool {
    value.starts_with('/') && !value.starts_with("//")
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests {
    use super::{ScriptLintResult, ScriptRule, VueRouterPreferNamedPush};

    #[test]
    fn accepts_named_route_object() {
        let mut result = ScriptLintResult::default();
        VueRouterPreferNamedPush.check("router.push({ name: 'home' })", 0, &mut result);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_string_path_push() {
        let mut result = ScriptLintResult::default();
        VueRouterPreferNamedPush.check("router.push('/users')", 0, &mut result);
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_object_path_replace() {
        let mut result = ScriptLintResult::default();
        VueRouterPreferNamedPush.check("router.replace({ path: '/settings' })", 0, &mut result);
        assert_eq!(result.warning_count, 1);
    }
}
