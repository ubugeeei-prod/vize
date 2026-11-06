//! script/require-define-component
//!
//! Require a TypeScript SFC's default export to be created with
//! `defineComponent()`.
//!
//! `defineComponent()` gives an Options API object contextual TypeScript
//! types for `this`, props, emits, and the other component options. A plain
//! object default export loses that component-specific checking. This rule is
//! deliberately scoped to ordinary `<script lang="ts">` / `lang="tsx"`
//! blocks: JavaScript components do not need the typing helper, and
//! `<script setup>` has no runtime default export.
//!
//! `defineNuxtComponent()` is accepted as Nuxt's typed equivalent. TypeScript
//! parentheses and assertions around either helper call are also accepted.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script lang="ts">
//! export default {
//!   props: { count: Number },
//!   mounted() {
//!     console.log(this.count)
//!   }
//! }
//! </script>
//! ```
//!
//! ### Valid
//! ```vue
//! <script lang="ts">
//! import { defineComponent } from 'vue'
//! export default defineComponent({
//!   props: { count: Number },
//!   mounted() {
//!     console.log(this.count)
//!   }
//! })
//! </script>
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{CallExpression, ExportDefaultDeclarationKind, Expression, Program, Statement};
use oxc_span::Span;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-define-component",
    description: "Require defineComponent() for TypeScript SFC default exports",
    default_severity: Severity::Error,
};

/// Require `defineComponent()` for a default export in an ordinary TypeScript
/// SFC script block.
pub struct RequireDefineComponent;

impl ScriptRule for RequireDefineComponent {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn runs_on_script_setup(&self) -> bool {
        false
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        self.check_program_with_sfc(program, source, offset, SfcScriptContext::default(), result);
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        if !sfc.is_sfc || sfc.is_script_setup || !sfc.typescript {
            return;
        }

        for statement in &program.body {
            let Statement::ExportDefaultDeclaration(export) = statement else {
                continue;
            };
            if !is_typed_component_export(&export.declaration) {
                report(export.span, offset, result);
            }
        }
    }
}

fn is_typed_component_export(declaration: &ExportDefaultDeclarationKind<'_>) -> bool {
    match declaration {
        ExportDefaultDeclarationKind::CallExpression(call) => is_component_helper_call(call),
        ExportDefaultDeclarationKind::ParenthesizedExpression(expression) => {
            is_typed_component_expression(&expression.expression)
        }
        ExportDefaultDeclarationKind::TSAsExpression(expression) => {
            is_typed_component_expression(&expression.expression)
        }
        ExportDefaultDeclarationKind::TSSatisfiesExpression(expression) => {
            is_typed_component_expression(&expression.expression)
        }
        ExportDefaultDeclarationKind::TSNonNullExpression(expression) => {
            is_typed_component_expression(&expression.expression)
        }
        _ => false,
    }
}

fn is_typed_component_expression(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::CallExpression(call) => is_component_helper_call(call),
        Expression::ParenthesizedExpression(expression) => {
            is_typed_component_expression(&expression.expression)
        }
        Expression::TSAsExpression(expression) => {
            is_typed_component_expression(&expression.expression)
        }
        Expression::TSSatisfiesExpression(expression) => {
            is_typed_component_expression(&expression.expression)
        }
        Expression::TSNonNullExpression(expression) => {
            is_typed_component_expression(&expression.expression)
        }
        _ => false,
    }
}

fn is_component_helper_call(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier)
            if matches!(
                identifier.name.as_str(),
                "defineComponent" | "defineNuxtComponent" | "_defineComponent"
            )
    )
}

fn report(span: Span, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(
        LintDiagnostic::error(
            META.name,
            "TypeScript SFC default exports must use `defineComponent()`",
            start,
            end,
        )
        .with_label("default export is not wrapped", start, end)
        .with_help(
            "Wrap the component options in `defineComponent({ ... })` so Vue can provide contextual TypeScript types.",
        ),
    );
}

#[cfg(test)]
#[path = "require_define_component_tests.rs"]
mod tests;
