//! script/define-macros-order
//!
//! Enforce a consistent order of the Vue compiler macros in `<script setup>`.
//!
//! The Vue style guide and `eslint-plugin-vue` recommend declaring the compiler
//! macros in a fixed order so every component reads the same way. This rule
//! enforces the canonical order
//! `defineOptions` → `defineModel` → `defineProps` → `defineEmits` → `defineSlots`
//! and additionally expects every macro to precede other top-level statements.
//!
//! A macro statement is a top-level expression statement or variable declaration
//! whose value is (possibly through `withDefaults(...)`) a call to one of the
//! recognised macros. Imports and type-only statements are ignored; the first
//! non-macro *runtime* statement marks where the macro block ends.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! // defineProps before defineModel (out of canonical order)
//! const props = defineProps<{ count: number }>()
//! const model = defineModel<string>()
//! ```
//!
//! ```ts
//! // a macro after a non-macro runtime statement
//! const value = ref(0)
//! const props = defineProps<{ count: number }>()
//! ```
//!
//! ### Valid
//! ```ts
//! defineOptions({ name: 'MyComponent' })
//! const model = defineModel<string>()
//! const props = defineProps<{ count: number }>()
//! const emit = defineEmits<{ change: [value: string] }>()
//! defineSlots<{ default(props: {}): any }>()
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Expression, Program, Statement};
use oxc_span::{GetSpan, Span};
use vize_carton::{CompactString, cstr};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/define-macros-order",
    description: "Enforce a consistent order of the Vue compiler macros in <script setup>",
    default_severity: Severity::Warning,
};

/// The compiler macros this rule orders, in their canonical order. A macro's
/// index in this slice is its rank: a lower rank must appear earlier in source.
const MACRO_ORDER: [&str; 5] = [
    "defineOptions",
    "defineModel",
    "defineProps",
    "defineEmits",
    "defineSlots",
];

/// Rank of a macro name within [`MACRO_ORDER`], or `None` if it is not ordered.
fn macro_rank(name: &str) -> Option<usize> {
    MACRO_ORDER.iter().position(|candidate| *candidate == name)
}

/// Enforce a consistent order of the Vue compiler macros in `<script setup>`.
pub struct DefineMacrosOrder;

impl ScriptRule for DefineMacrosOrder {
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
        // Walk the top-level statements once, recording every macro call and the
        // first non-macro *runtime* statement. Every macro should sit at the top
        // of the block, so any runtime statement (even one before the first
        // macro) marks a boundary that no macro may appear after.
        let mut macros: Vec<MacroOccurrence> = Vec::new();
        let mut first_runtime: Option<Span> = None;

        for statement in &program.body {
            if let Some((rank, span)) = macro_occurrence(statement) {
                macros.push(MacroOccurrence { rank, span });
            } else if first_runtime.is_none() && is_runtime_statement(statement) {
                first_runtime = Some(statement.span());
            }
        }

        if macros.is_empty() {
            return;
        }

        report_out_of_order(&macros, offset, result);
        report_after_non_macro(&macros, first_runtime, offset, result);
    }
}

/// A recognised compiler-macro call found at the top level.
struct MacroOccurrence {
    /// Rank within [`MACRO_ORDER`].
    rank: usize,
    /// Source span of the whole statement (used for diagnostics).
    span: Span,
}

/// Report each macro that appears after a macro which should come later per the
/// canonical order. We compare every macro against the maximum rank seen so far:
/// a macro whose rank is *strictly less* than an earlier macro's rank is out of
/// order (e.g. `defineProps` (rank 2) appearing after `defineEmits` (rank 3)).
fn report_out_of_order(macros: &[MacroOccurrence], offset: usize, result: &mut ScriptLintResult) {
    let mut max_rank_so_far = macros[0].rank;
    for occurrence in &macros[1..] {
        if occurrence.rank < max_rank_so_far {
            let expected = MACRO_ORDER[occurrence.rank];
            let after = MACRO_ORDER[max_rank_so_far];
            report(
                occurrence.span,
                offset,
                cstr!(
                    "`{expected}` should be declared before `{after}` to follow the \
                     canonical macro order."
                ),
                result,
            );
        } else {
            max_rank_so_far = occurrence.rank;
        }
    }
}

/// Report any macro that appears after the first non-macro runtime statement.
fn report_after_non_macro(
    macros: &[MacroOccurrence],
    first_runtime: Option<Span>,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let Some(boundary) = first_runtime else {
        return;
    };
    for occurrence in macros {
        if occurrence.span.start > boundary.start {
            let name = MACRO_ORDER[occurrence.rank];
            report(
                occurrence.span,
                offset,
                cstr!(
                    "`{name}` should be declared before other statements in \
                     `<script setup>`."
                ),
                result,
            );
        }
    }
}

fn report(span: Span, offset: usize, message: CompactString, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(
        LintDiagnostic::warn(META.name, message, start, end)
            .with_label("compiler macro out of order", start, end)
            .with_help(
                "Declare the Vue compiler macros in the canonical order: \
                 `defineOptions`, `defineModel`, `defineProps`, `defineEmits`, \
                 `defineSlots`, before any other statement.",
            ),
    );
}

/// The macro rank and statement span if `statement` is a recognised top-level
/// compiler-macro call.
fn macro_occurrence(statement: &Statement<'_>) -> Option<(usize, Span)> {
    match statement {
        Statement::ExpressionStatement(stmt) => {
            macro_rank_of_expression(&stmt.expression).map(|rank| (rank, stmt.span))
        }
        Statement::VariableDeclaration(decl) => {
            // `const props = defineProps(...)` — inspect each declarator's init,
            // reporting on the whole declaration statement.
            decl.declarations
                .iter()
                .find_map(|declarator| declarator.init.as_ref().and_then(macro_rank_of_expression))
                .map(|rank| (rank, decl.span))
        }
        _ => None,
    }
}

/// The macro rank of an expression, unwrapping a single `withDefaults(...)`
/// wrapper (`withDefaults(defineProps(...), {})`) to its inner macro call.
fn macro_rank_of_expression(expression: &Expression<'_>) -> Option<usize> {
    let Expression::CallExpression(call) = expression else {
        return None;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    let name = callee.name.as_str();

    if name == "withDefaults" {
        // The macro is the first argument of `withDefaults(defineProps(...), …)`.
        return call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
            .and_then(macro_rank_of_expression);
    }

    macro_rank(name)
}

/// Whether a statement is a runtime statement that should not appear in the
/// middle of the macro block. Imports and TS type-only declarations are exempt
/// because they are hoisted / erased and conventionally precede the macros.
fn is_runtime_statement(statement: &Statement<'_>) -> bool {
    !matches!(
        statement,
        Statement::ImportDeclaration(_)
            | Statement::TSTypeAliasDeclaration(_)
            | Statement::TSInterfaceDeclaration(_)
            | Statement::TSModuleDeclaration(_)
            | Statement::TSImportEqualsDeclaration(_)
            | Statement::TSExportAssignment(_)
            | Statement::EmptyStatement(_)
    )
}

#[cfg(test)]
mod tests;
