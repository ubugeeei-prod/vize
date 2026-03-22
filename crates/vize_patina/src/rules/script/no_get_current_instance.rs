//! script/no-get-current-instance
//!
//! Disallow `getCurrentInstance()` in Vapor mode.
//!
//! In Vapor mode, `getCurrentInstance()` returns `null`. Code that relies
//! on this function will not work correctly in Vapor components.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! import { getCurrentInstance } from 'vue'
//!
//! const instance = getCurrentInstance()
//! const proxy = instance?.proxy
//! ```
//!
//! ### Valid
//! ```ts
//! import { useAttrs, useSlots } from 'vue'
//!
//! const attrs = useAttrs()
//! const slots = useSlots()
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_allocator::Allocator;
use oxc_ast::ast::{CallExpression, Expression, ImportDeclaration, ImportDeclarationSpecifier};
use oxc_ast_visit::{
    walk::{walk_call_expression, walk_import_declaration},
    Visit,
};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use vize_carton::{CompactString, FxHashSet};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-get-current-instance",
    description: "Disallow getCurrentInstance() in Vapor mode (returns null)",
    default_severity: Severity::Error,
};

/// Disallow getCurrentInstance()
pub struct NoGetCurrentInstance;

impl ScriptRule for NoGetCurrentInstance {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn check(&self, source: &str, offset: usize, result: &mut ScriptLintResult) {
        let allocator = Allocator::default();
        let source_type =
            SourceType::from_path("component.ts").unwrap_or_else(|_| SourceType::ts());
        let parsed = Parser::new(&allocator, source, source_type).parse();
        if parsed.panicked || !parsed.errors.is_empty() {
            return;
        }

        let mut visitor = NoGetCurrentInstanceVisitor {
            offset,
            result,
            imported_aliases: FxHashSet::default(),
            namespace_aliases: FxHashSet::default(),
        };
        visitor.visit_program(&parsed.program);
    }
}

struct NoGetCurrentInstanceVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    imported_aliases: FxHashSet<CompactString>,
    namespace_aliases: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for NoGetCurrentInstanceVisitor<'_> {
    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        if is_vue_module(it.source.value.as_str()) {
            if let Some(specifiers) = &it.specifiers {
                for specifier in specifiers {
                    match specifier {
                        ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                            if specifier.imported.name().as_str() != "getCurrentInstance" {
                                continue;
                            }

                            self.imported_aliases
                                .insert(CompactString::new(specifier.local.name.as_str()));
                            self.push_diagnostic(
                                specifier.local.span,
                                "getCurrentInstance import is not supported in Vapor-oriented components",
                                "Remove getCurrentInstance() usage and replace instance access with explicit Composition API state such as useAttrs(), useSlots(), provide/inject, or template refs.",
                            );
                        }
                        ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                            self.namespace_aliases
                                .insert(CompactString::new(specifier.local.name.as_str()));
                        }
                        ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {}
                    }
                }
            }
        }

        walk_import_declaration(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(span) =
            get_current_instance_call_span(it, &self.imported_aliases, &self.namespace_aliases)
        {
            self.push_diagnostic(
                span,
                "getCurrentInstance() is not supported in Vapor-oriented components",
                "Avoid instance-level access in Vapor mode because getCurrentInstance() returns null. Prefer explicit Composition API bindings, provide/inject, template refs, or lifecycle hooks instead.",
            );
        }

        walk_call_expression(self, it);
    }
}

impl NoGetCurrentInstanceVisitor<'_> {
    fn push_diagnostic(&mut self, span: Span, message: &'static str, help: &'static str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result
            .add_diagnostic(LintDiagnostic::error(META.name, message, start, end).with_help(help));
    }
}

#[inline]
fn is_vue_module(source: &str) -> bool {
    source == "vue" || source.starts_with("@vue/")
}

fn get_current_instance_call_span(
    call: &CallExpression<'_>,
    imported_aliases: &FxHashSet<CompactString>,
    namespace_aliases: &FxHashSet<CompactString>,
) -> Option<Span> {
    match &call.callee {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if name == "getCurrentInstance" || imported_aliases.contains(name) {
                return Some(identifier.span);
            }
            None
        }
        expression if expression.is_member_expression() => expression
            .as_member_expression()
            .filter(|member| member.static_property_name() == Some("getCurrentInstance"))
            .and_then(|member| expression_identifier_name(member.object()))
            .filter(|name| namespace_aliases.contains(*name))
            .map(|_| expression.span()),
        Expression::ParenthesizedExpression(paren) => get_current_instance_expression_span(
            &paren.expression,
            imported_aliases,
            namespace_aliases,
        ),
        Expression::TSAsExpression(ts_as) => get_current_instance_expression_span(
            &ts_as.expression,
            imported_aliases,
            namespace_aliases,
        ),
        Expression::TSSatisfiesExpression(ts_satisfies) => get_current_instance_expression_span(
            &ts_satisfies.expression,
            imported_aliases,
            namespace_aliases,
        ),
        Expression::TSNonNullExpression(ts_non_null) => get_current_instance_expression_span(
            &ts_non_null.expression,
            imported_aliases,
            namespace_aliases,
        ),
        _ => None,
    }
}

fn get_current_instance_expression_span(
    expression: &Expression<'_>,
    imported_aliases: &FxHashSet<CompactString>,
    namespace_aliases: &FxHashSet<CompactString>,
) -> Option<Span> {
    match expression {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if name == "getCurrentInstance" || imported_aliases.contains(name) {
                Some(identifier.span)
            } else {
                None
            }
        }
        member if member.is_member_expression() => member
            .as_member_expression()
            .filter(|member| member.static_property_name() == Some("getCurrentInstance"))
            .and_then(|member| expression_identifier_name(member.object()))
            .filter(|name| namespace_aliases.contains(*name))
            .map(|_| member.span()),
        Expression::ParenthesizedExpression(paren) => get_current_instance_expression_span(
            &paren.expression,
            imported_aliases,
            namespace_aliases,
        ),
        Expression::TSAsExpression(ts_as) => get_current_instance_expression_span(
            &ts_as.expression,
            imported_aliases,
            namespace_aliases,
        ),
        Expression::TSSatisfiesExpression(ts_satisfies) => get_current_instance_expression_span(
            &ts_satisfies.expression,
            imported_aliases,
            namespace_aliases,
        ),
        Expression::TSNonNullExpression(ts_non_null) => get_current_instance_expression_span(
            &ts_non_null.expression,
            imported_aliases,
            namespace_aliases,
        ),
        _ => None,
    }
}

fn expression_identifier_name<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    match expression {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::ParenthesizedExpression(paren) => expression_identifier_name(&paren.expression),
        Expression::TSAsExpression(ts_as) => expression_identifier_name(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            expression_identifier_name(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            expression_identifier_name(&ts_non_null.expression)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{NoGetCurrentInstance, ScriptLintResult, ScriptRule};

    #[test]
    fn test_valid_no_get_current_instance() {
        let source = r#"
import { ref, useAttrs } from 'vue'
const count = ref(0)
const attrs = useAttrs()
"#;
        let rule = NoGetCurrentInstance;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_get_current_instance_import() {
        let source = r#"
import { getCurrentInstance } from 'vue'
const instance = getCurrentInstance()
"#;
        let rule = NoGetCurrentInstance;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_invalid_aliased_get_current_instance_usage() {
        let source = r#"
import { getCurrentInstance as useInstance } from 'vue'
const instance = useInstance()
"#;
        let rule = NoGetCurrentInstance;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_invalid_namespace_get_current_instance_usage() {
        let source = r#"
import * as Vue from 'vue'
const instance = Vue.getCurrentInstance()
"#;
        let rule = NoGetCurrentInstance;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_auto_import_style_usage() {
        let source = "const proxy = getCurrentInstance()?.proxy";
        let rule = NoGetCurrentInstance;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0].message.contains("getCurrentInstance"));
    }
}
