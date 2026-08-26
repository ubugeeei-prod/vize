//! ecosystem/pinia-prefer-store-to-refs
//!
//! Warn when Pinia stores are destructured directly.
//!
//! Direct store destructuring disconnects state/getters from Pinia's reactive
//! proxy. `storeToRefs()` keeps state and getters reactive while actions stay on
//! the store instance.

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use memchr::memmem;
use oxc_ast::ast::{
    BindingPattern, CallExpression, Declaration, Expression, Program, Statement,
    VariableDeclaration, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk::walk_variable_declarator};
use oxc_span::{GetSpan, Span};
use vize_s0::{CompactString, FxHashSet};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "ecosystem/pinia-prefer-store-to-refs",
    description: "Prefer storeToRefs() when destructuring Pinia stores",
    default_severity: Severity::Warning,
};

pub struct PiniaPreferStoreToRefs;

impl ScriptRule for PiniaPreferStoreToRefs {
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
        if memmem::find(source.as_bytes(), b"Store").is_none() {
            return;
        }

        let local_callables = collect_local_callables(program);

        let mut visitor = PiniaVisitor {
            local_callables: &local_callables,
            offset,
            result,
            store_bindings: FxHashSet::default(),
        };
        visitor.visit_program(program);
    }
}

struct PiniaVisitor<'result> {
    local_callables: &'result FxHashSet<CompactString>,
    offset: usize,
    result: &'result mut ScriptLintResult,
    store_bindings: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for PiniaVisitor<'_> {
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'a>) {
        let Some(init) = it.init.as_ref() else {
            walk_variable_declarator(self, it);
            return;
        };

        match &it.id {
            BindingPattern::BindingIdentifier(identifier)
                if is_pinia_store_expression(init, self.local_callables) =>
            {
                self.store_bindings
                    .insert(CompactString::new(identifier.name.as_str()));
            }
            BindingPattern::ObjectPattern(pattern) if self.is_store_destructure(init) => {
                self.push_diagnostic(pattern.span());
            }
            _ => {}
        }

        walk_variable_declarator(self, it);
    }
}

impl PiniaVisitor<'_> {
    fn is_store_destructure(&self, expression: &Expression<'_>) -> bool {
        if is_pinia_store_expression(expression, self.local_callables) {
            return true;
        }

        expression_identifier_name(expression)
            .is_some_and(|name| self.store_bindings.contains(name))
    }

    fn push_diagnostic(&mut self, span: Span) {
        self.result.add_diagnostic(
            LintDiagnostic::warn(
                META.name,
                "Pinia stores should not be destructured directly",
                self.offset as u32 + span.start,
                self.offset as u32 + span.end,
            )
            .with_help(
                "Use `storeToRefs(store)` for state/getters and keep actions on the store object.",
            ),
        );
    }
}

fn collect_local_callables(program: &Program<'_>) -> FxHashSet<CompactString> {
    let mut names = FxHashSet::default();
    for statement in &program.body {
        match statement {
            Statement::FunctionDeclaration(function) => {
                if let Some(identifier) = &function.id {
                    names.insert(CompactString::new(identifier.name.as_str()));
                }
            }
            Statement::VariableDeclaration(declaration) => {
                collect_callable_variables(declaration, &mut names);
            }
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::FunctionDeclaration(function)) => {
                    if let Some(identifier) = &function.id {
                        names.insert(CompactString::new(identifier.name.as_str()));
                    }
                }
                Some(Declaration::VariableDeclaration(declaration)) => {
                    collect_callable_variables(declaration, &mut names);
                }
                _ => {}
            },
            _ => {}
        }
    }
    names
}

fn collect_callable_variables(
    declaration: &VariableDeclaration<'_>,
    names: &mut FxHashSet<CompactString>,
) {
    for declarator in &declaration.declarations {
        if let BindingPattern::BindingIdentifier(identifier) = &declarator.id
            && declarator.init.as_ref().is_some_and(is_function_expression)
        {
            names.insert(CompactString::new(identifier.name.as_str()));
        }
    }
}

fn is_function_expression(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => true,
        Expression::ParenthesizedExpression(paren) => is_function_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => is_function_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            is_function_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            is_function_expression(&ts_non_null.expression)
        }
        _ => false,
    }
}

fn is_pinia_store_expression(
    expression: &Expression<'_>,
    local_callables: &FxHashSet<CompactString>,
) -> bool {
    match expression {
        Expression::CallExpression(call) => is_pinia_store_call(call, local_callables),
        Expression::ParenthesizedExpression(paren) => {
            is_pinia_store_expression(&paren.expression, local_callables)
        }
        Expression::TSAsExpression(ts_as) => {
            is_pinia_store_expression(&ts_as.expression, local_callables)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            is_pinia_store_expression(&ts_satisfies.expression, local_callables)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            is_pinia_store_expression(&ts_non_null.expression, local_callables)
        }
        _ => false,
    }
}

fn is_pinia_store_call(
    call: &CallExpression<'_>,
    local_callables: &FxHashSet<CompactString>,
) -> bool {
    match &call.callee {
        Expression::Identifier(identifier) => {
            let name = identifier.name.as_str();
            is_store_composable_name(name) && !local_callables.contains(name)
        }
        expression if expression.is_member_expression() => expression
            .as_member_expression()
            .and_then(|member| member.static_property_name())
            .is_some_and(is_store_composable_name),
        _ => false,
    }
}

fn is_store_composable_name(name: &str) -> bool {
    name.len() > "useStore".len() && name.starts_with("use") && name.ends_with("Store")
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
#[allow(clippy::disallowed_macros)]
mod tests {
    use super::{PiniaPreferStoreToRefs, ScriptLintResult, ScriptRule};

    #[test]
    fn accepts_store_to_refs() {
        let source = r#"
const store = useUserStore()
const { name } = storeToRefs(store)
"#;
        let mut result = ScriptLintResult::default();
        PiniaPreferStoreToRefs.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_direct_store_call_destructure() {
        let source = "const { name } = useUserStore()";
        let mut result = ScriptLintResult::default();
        PiniaPreferStoreToRefs.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn reports_store_binding_destructure() {
        let source = r#"
const store = useUserStore()
const { name } = store
"#;
        let mut result = ScriptLintResult::default();
        PiniaPreferStoreToRefs.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn accepts_same_file_function_named_like_a_store() {
        let source = r#"
export function useCounterStore() {
    return { count: ref(0), inc: () => {} }
}
const { count, inc } = useCounterStore()
"#;
        let mut result = ScriptLintResult::default();
        PiniaPreferStoreToRefs.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn accepts_same_file_function_expression_named_like_a_store() {
        let source = r#"
const useCounterStore = () => ({ count: ref(0) })
const counter = useCounterStore()
const { count } = counter
"#;
        let mut result = ScriptLintResult::default();
        PiniaPreferStoreToRefs.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn reports_actual_define_store_factory_declared_in_the_same_file() {
        let source = r#"
const useUserStore = defineStore('user', { state: () => ({ name: '' }) })
const { name } = useUserStore()
"#;
        let mut result = ScriptLintResult::default();
        PiniaPreferStoreToRefs.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 1);
    }
}
