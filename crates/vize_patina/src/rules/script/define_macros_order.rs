//! script/define-macros-order
//!
//! Enforce the order of the Vue compiler macros in `<script setup>`.
//!
//! Matches eslint-plugin-vue's `vue/define-macros-order`: the configured macros
//! must be the first statements of `<script setup>`, after imports, type
//! declarations, enums and exports. The default order is `defineProps` and then
//! `defineEmits`; other macros count as ordinary statements unless configured.
//! Prefix order reports at most once; `defineExposeLast` adds a setup-only last check.
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

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
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
        ConfiguredDefineMacrosOrder(options)
    }
}

struct ConfiguredDefineMacrosOrder(vize_l0::config::DefineMacrosOrderOptions);

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
        check(program, offset, result, &DEFAULT_MACRO_ORDER, false);
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
        let order = self
            .0
            .order
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>();
        let expose_last = self.0.define_expose_last && sfc.is_sfc && sfc.is_script_setup;
        let own_start = result.diagnostics.len();
        check(program, offset, result, &order, expose_last);
        if expose_last && let Some(own) = result.diagnostics.get_mut(own_start..) {
            // ESLint presents these independent findings in authored source order.
            // Keep other rules' diagnostics and the default-off path untouched.
            own.sort_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
        }
    }
}

fn check<'a>(
    program: &'a Program<'a>,
    offset: usize,
    result: &mut ScriptLintResult,
    macro_order: &[&str],
    define_expose_last: bool,
) {
    let body = &program.body;
    // Upstream emits this suggestion-only finding before the ordinary order
    // finding. Trailing type/import/empty statements still occupy the last slot.
    if define_expose_last
        && let Some((index, statement)) = body
            .iter()
            .enumerate()
            .rfind(|(_, statement)| is_define_expose_statement(statement))
        && index + 1 < body.len()
    {
        let span = statement.span();
        let start = offset as u32 + span.start;
        let end = offset as u32 + span.end;
        result.add_diagnostic(
            LintDiagnostic::warn(
                META.name,
                "`defineExpose` should be the last statement in `<script setup>`.",
                start,
                end,
            )
            .with_label("compiler macro should be last", start, end)
            .with_help("Put `defineExpose` as the last statement in `<script setup>`."),
        );
    }
    let first = body
        .iter()
        .position(|statement| !is_skipped_statement(statement))
        .unwrap_or(body.len());
    // Every occurrence occupies a slot in its configured group. Keeping the
    // authored order within each group accepts repeated calls without hiding a
    // group split by another macro or an ordinary statement. No list is needed.
    let mut target = first;
    for (group, &name) in macro_order.iter().enumerate() {
        // Deserialized options are unique; typed callers can also construct them.
        if macro_order
            .iter()
            .take(group)
            .any(|previous| *previous == name)
        {
            continue;
        }
        for (index, statement) in body.iter().enumerate() {
            if macro_name(statement) != Some(name) {
                continue;
            }
            if index == target {
                target += 1;
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
            report(statement.span(), offset, message, result);
            return;
        }
    }
}

fn is_define_expose_statement(statement: &Statement<'_>) -> bool {
    let direct = |expression: &Expression<'_>| {
        matches!(expression, Expression::CallExpression(call)
            if matches!(&call.callee, Expression::Identifier(name) if name.name == "defineExpose"))
    };
    match statement {
        Statement::ExpressionStatement(statement) => direct(&statement.expression),
        Statement::VariableDeclaration(declaration) if !declaration.declare => declaration
            .declarations
            .iter()
            .any(|value| value.init.as_ref().is_some_and(direct)),
        _ => false,
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
        // Ambient declarations do not execute a macro, including initializers
        // retained by the parser while a TypeScript buffer is being edited.
        Statement::VariableDeclaration(decl) if !decl.declare => decl
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
