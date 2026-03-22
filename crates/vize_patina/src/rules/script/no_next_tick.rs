//! script/no-next-tick
//!
//! Disallow `nextTick()` in Vapor-oriented components.
//!
//! Vapor-oriented code should avoid DOM-flush timing dependencies. `nextTick()`
//! and `this.$nextTick()` are migration smells because they usually indicate the
//! component still depends on VDOM-style post-render scheduling.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! import { nextTick } from 'vue'
//!
//! await nextTick()
//! this.$nextTick(() => focusInput())
//! ```
//!
//! ### Valid
//! ```ts
//! import { onMounted } from 'vue'
//!
//! onMounted(() => {
//!   focusInput()
//! })
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
    name: "script/no-next-tick",
    description: "Disallow nextTick() usage in Vapor-oriented components",
    default_severity: Severity::Error,
};

/// Disallow nextTick()
pub struct NoNextTick;

impl ScriptRule for NoNextTick {
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

        let mut visitor = NoNextTickVisitor {
            offset,
            result,
            imported_aliases: FxHashSet::default(),
        };
        visitor.visit_program(&parsed.program);
    }
}

struct NoNextTickVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    imported_aliases: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for NoNextTickVisitor<'_> {
    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        if it.source.value.as_str() == "vue" {
            if let Some(specifiers) = &it.specifiers {
                for specifier in specifiers {
                    let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                        continue;
                    };

                    if specifier.imported.name().as_str() != "nextTick" {
                        continue;
                    }

                    self.imported_aliases
                        .insert(CompactString::new(specifier.local.name.as_str()));
                    self.push_diagnostic(
                        specifier.local.span,
                        "nextTick import is not supported in Vapor-oriented components",
                        "Remove nextTick() usage and rely on explicit lifecycle boundaries like onMounted() or direct reactive flow instead.",
                    );
                }
            }
        }

        walk_import_declaration(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(span) = next_tick_call_span(it, &self.imported_aliases) {
            self.push_diagnostic(
                span,
                "nextTick() is not supported in Vapor-oriented components",
                "Avoid post-render scheduling with nextTick(). Prefer onMounted(), template refs, or a control flow that does not depend on a DOM flush boundary.",
            );
        }

        walk_call_expression(self, it);
    }
}

impl NoNextTickVisitor<'_> {
    fn push_diagnostic(&mut self, span: Span, message: &'static str, help: &'static str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result
            .add_diagnostic(LintDiagnostic::error(META.name, message, start, end).with_help(help));
    }
}

fn next_tick_call_span(
    call: &CallExpression<'_>,
    imported_aliases: &FxHashSet<CompactString>,
) -> Option<Span> {
    match &call.callee {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if name == "nextTick" || name == "$nextTick" || imported_aliases.contains(name) {
                return Some(identifier.span);
            }
            None
        }
        expression if expression.is_member_expression() => expression
            .as_member_expression()
            .and_then(|member| member.static_property_name())
            .filter(|name| *name == "$nextTick")
            .map(|_| expression.span()),
        Expression::ParenthesizedExpression(paren) => {
            next_tick_expression_span(&paren.expression, imported_aliases)
        }
        Expression::TSAsExpression(ts_as) => {
            next_tick_expression_span(&ts_as.expression, imported_aliases)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            next_tick_expression_span(&ts_satisfies.expression, imported_aliases)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            next_tick_expression_span(&ts_non_null.expression, imported_aliases)
        }
        _ => None,
    }
}

fn next_tick_expression_span(
    expression: &Expression<'_>,
    imported_aliases: &FxHashSet<CompactString>,
) -> Option<Span> {
    match expression {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            if name == "nextTick" || name == "$nextTick" || imported_aliases.contains(name) {
                Some(identifier.span)
            } else {
                None
            }
        }
        member if member.is_member_expression() => member
            .as_member_expression()
            .and_then(|member| member.static_property_name())
            .filter(|name| *name == "$nextTick")
            .map(|_| member.span()),
        Expression::ParenthesizedExpression(paren) => {
            next_tick_expression_span(&paren.expression, imported_aliases)
        }
        Expression::TSAsExpression(ts_as) => {
            next_tick_expression_span(&ts_as.expression, imported_aliases)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            next_tick_expression_span(&ts_satisfies.expression, imported_aliases)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            next_tick_expression_span(&ts_non_null.expression, imported_aliases)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{NoNextTick, ScriptLintResult, ScriptRule};

    #[test]
    fn test_valid_without_next_tick() {
        let source = r#"
import { onMounted } from 'vue'
onMounted(() => focusInput())
"#;
        let rule = NoNextTick;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_import_and_usage() {
        let source = r#"
import { nextTick } from 'vue'
await nextTick()
"#;
        let rule = NoNextTick;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_invalid_aliased_import_usage() {
        let source = r#"
import { nextTick as afterRender } from 'vue'
await afterRender()
"#;
        let rule = NoNextTick;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_invalid_instance_next_tick() {
        let source = "this.$nextTick(() => focusInput())";
        let rule = NoNextTick;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_nuxt_auto_import_style_call() {
        let source = "void nextTick(() => focusInput())";
        let rule = NoNextTick;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
    }
}
