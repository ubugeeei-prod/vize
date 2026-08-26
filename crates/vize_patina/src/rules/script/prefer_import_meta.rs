//! nuxt/prefer-import-meta
//!
//! Prefer Nuxt's `import.meta.*` runtime flags over the legacy `process.*`
//! aliases. This ports `@nuxt/eslint-plugin` 1.16.0's rule exactly: only the
//! seven upstream suffixes match, computed identifier properties match, and
//! lexical shadowing of `process` does not suppress the diagnostic.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use oxc_ast::ast::{ComputedMemberExpression, Expression, Program, StaticMemberExpression};
use oxc_ast_visit::{
    Visit,
    walk::{walk_computed_member_expression, walk_static_member_expression},
};
use oxc_span::Span;
use vize_s0::cstr;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "nuxt/prefer-import-meta",
    description: "Prefer using `import.meta.*` over `process.*`",
    default_severity: Severity::Error,
};

const PROCESS_SUFFIXES: &[&str] = &[
    "client",
    "browser",
    "server",
    "nitro",
    "dev",
    "test",
    "prerender",
];

/// Prefer `import.meta.*` over Nuxt's legacy `process.*` runtime flags.
pub struct PreferImportMeta;

impl ScriptRule for PreferImportMeta {
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
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let mut visitor = PreferImportMetaVisitor { offset, result };
        visitor.visit_program(program);
    }
}

struct PreferImportMetaVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for PreferImportMetaVisitor<'_> {
    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if is_process_identifier(&member.object) {
            self.report(member.span, member.property.name.as_str());
        }
        walk_static_member_expression(self, member);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if is_process_identifier(&member.object)
            && let Expression::Identifier(property) = &member.expression
        {
            self.report(member.span, property.name.as_str());
        }
        walk_computed_member_expression(self, member);
    }
}

fn is_process_identifier(expression: &Expression<'_>) -> bool {
    matches!(expression, Expression::Identifier(identifier) if identifier.name == "process")
}

impl PreferImportMetaVisitor<'_> {
    fn report(&mut self, span: Span, suffix: &str) {
        if !PROCESS_SUFFIXES.contains(&suffix) {
            return;
        }

        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        let replacement = cstr!("import.meta.{suffix}");
        self.result.add_diagnostic(
            LintDiagnostic::error(
                META.name,
                cstr!("Replace `process.{suffix}` with `import.meta.{suffix}`."),
                start,
                end,
            )
            .with_help(cstr!("Use `import.meta.{suffix}` instead."))
            .with_fix(Fix::new(
                cstr!("Replace with `import.meta.{suffix}`"),
                TextEdit::replace(start, end, replacement),
            )),
        );
    }
}

#[cfg(test)]
mod tests;
