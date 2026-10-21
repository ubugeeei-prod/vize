//! script/define-macros-order
//!
//! Enforce the order of the Vue compiler macros in `<script setup>`.
//!
//! Matches eslint-plugin-vue's `vue/define-macros-order`: the configured macros
//! must be the first statements of `<script setup>`, after imports, type
//! declarations, enums and exports. The default order is `defineProps` and then
//! `defineEmits`; other macros count as ordinary statements unless configured.
//! Like the upstream rule, it reports at most one statement per block.
//!
//! A macro statement is a top-level expression statement or variable declaration
//! whose value is (possibly through `withDefaults(...)`) a call to the macro.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! // defineEmits before defineProps
//! const emit = defineEmits<{ change: [value: string] }>()
//! const props = defineProps<{ count: number }>()
//! ```
//!
//! ```ts
//! // a statement before the macros
//! const value = ref(0)
//! const props = defineProps<{ count: number }>()
//! ```
//!
//! ### Valid
//! ```ts
//! import { ref } from 'vue'
//! const props = defineProps<{ count: number }>()
//! const emit = defineEmits<{ change: [value: string] }>()
//! const model = defineModel<string>()
//! const value = ref(0)
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Expression, Program, Statement};
use oxc_span::{GetSpan, Span};
use vize_l0::{CompactString, cstr};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/define-macros-order",
    description: "Enforce a consistent order of the Vue compiler macros in <script setup>",
    default_severity: Severity::Warning,
};

/// eslint-plugin-vue's default `order` option.
const DEFAULT_MACRO_ORDER: [&str; 2] = ["defineProps", "defineEmits"];

/// Enforce the order of the Vue compiler macros in `<script setup>`.
pub struct DefineMacrosOrder;

impl DefineMacrosOrder {
    pub fn configured(options: vize_l0::config::DefineMacrosOrderOptions) -> impl ScriptRule {
        ConfiguredDefineMacrosOrder(options.order)
    }
}

struct ConfiguredDefineMacrosOrder(Vec<vize_l0::String>);

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
        check(program, offset, result, &DEFAULT_MACRO_ORDER);
    }
}

impl ScriptRule for ConfiguredDefineMacrosOrder {
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
        let order = self.0.iter().map(|name| name.as_str()).collect::<Vec<_>>();
        check(program, offset, result, &order);
    }
}

fn check<'a>(
    program: &'a Program<'a>,
    offset: usize,
    result: &mut ScriptLintResult,
    macro_order: &[&str],
) {
    let body = &program.body;
    // The last declaration of each ordered macro, in the expected order.
    let ordered: Vec<(&str, usize)> = macro_order
        .iter()
        .filter_map(|name| {
            body.iter()
                .rposition(|statement| macro_name(statement) == Some(name))
                .map(|index| (*name, index))
        })
        .collect();
    if ordered.is_empty() {
        return;
    }
    let first = body
        .iter()
        .position(|statement| !is_skipped_statement(statement))
        .unwrap_or(body.len());
    for (position, &(name, index)) in ordered.iter().enumerate() {
        let target = first + position;
        if index == target {
            continue;
        }
        let before = body
            .get(target)
            .and_then(macro_name)
            .filter(|macro_name| macro_order.contains(macro_name));
        let message = match before {
            Some(before) => cstr!("`{name}` should be above `{before}`."),
            None => cstr!(
                "`{name}` should be placed at the top of `<script setup>` (after any \
                     potential import statements or type definitions)."
            ),
        };
        if let Some(statement) = body.get(index) {
            report(statement.span(), offset, message, result);
        }
        return;
    }
}

fn report(span: Span, offset: usize, message: CompactString, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(
        LintDiagnostic::warn(META.name, message, start, end)
            .with_label("compiler macro out of order", start, end)
            .with_help(
                "Declare `defineProps` and then `defineEmits` first in `<script setup>`, \
                 right after imports and type declarations.",
            ),
    );
}

/// The compiler macro a top-level statement declares, if any.
fn macro_name<'s>(statement: &Statement<'s>) -> Option<&'s str> {
    match statement {
        Statement::ExpressionStatement(stmt) => macro_of_expression(&stmt.expression),
        // `const props = defineProps(...)`: any declarator may hold the macro.
        Statement::VariableDeclaration(decl) => decl
            .declarations
            .iter()
            .find_map(|declarator| declarator.init.as_ref().and_then(macro_of_expression)),
        _ => None,
    }
}

/// The macro an expression calls, looking through one `withDefaults(...)`
/// wrapper (`withDefaults(defineProps(...), {})`).
fn macro_of_expression<'s>(expression: &Expression<'s>) -> Option<&'s str> {
    let Expression::CallExpression(call) = expression else {
        return None;
    };
    let Expression::Identifier(callee) = &call.callee else {
        return None;
    };
    let name = callee.name.as_str();
    if name == "withDefaults" {
        return call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
            .and_then(macro_of_expression);
    }
    Some(name)
}

/// Statements that may precede the macros, as in eslint-plugin-vue: imports,
/// type declarations, enums, exports, `declare` statements, `debugger`, empty
/// statements and a `"use strict"` directive.
fn is_skipped_statement(statement: &Statement<'_>) -> bool {
    match statement {
        Statement::ImportDeclaration(_)
        | Statement::TSEnumDeclaration(_)
        | Statement::TSModuleDeclaration(_)
        | Statement::TSInterfaceDeclaration(_)
        | Statement::TSTypeAliasDeclaration(_)
        | Statement::DebuggerStatement(_)
        | Statement::EmptyStatement(_)
        | Statement::ExportNamedDeclaration(_) => true,
        Statement::VariableDeclaration(decl) => decl.declare,
        Statement::ClassDeclaration(class) => class.declare,
        Statement::FunctionDeclaration(function) => function.declare,
        Statement::ExpressionStatement(stmt) => matches!(
            &stmt.expression,
            Expression::StringLiteral(literal) if literal.value == "use strict"
        ),
        _ => false,
    }
}

#[cfg(test)]
mod tests;
