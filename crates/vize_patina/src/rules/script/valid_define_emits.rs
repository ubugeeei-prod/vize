//! script/valid-define-emits
//!
//! Enforce valid usage of the `defineEmits` compiler macro in `<script setup>`.
//!
//! `defineEmits` declares the events a component can emit, either via a runtime
//! argument (`defineEmits(['change'])`) or a type argument
//! (`defineEmits<{ change: [id: number] }>()`). It has three constraints, each
//! enforced here:
//!
//! 1. It may be called **at most once** per `<script setup>` block.
//! 2. It may not be given **both** a type argument and a runtime argument: the
//!    runtime and type declarations are mutually exclusive.
//! 3. Its runtime argument may not **reference locally-declared variables**:
//!    `defineEmits` is hoisted above the component setup, so any binding declared
//!    in the same block is not yet initialized when the macro runs.
//!
//! Mirrors [`vue/valid-define-emits`](https://eslint.vuejs.org/rules/valid-define-emits.html).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const local = ['change']
//! defineEmits(local)                       // references a local variable
//! defineEmits<Emits>(['change'])           // both type and runtime args
//! defineEmits(['change'])
//! defineEmits(['update'])                  // called more than once
//! ```
//!
//! ### Valid
//! ```ts
//! defineEmits(['change', 'update'])
//! defineEmits<{ change: [id: number] }>()
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    BindingPattern, CallExpression, Expression, IdentifierReference, Program, Statement,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::Span;
use vize_carton::FxHashSet;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/valid-define-emits",
    description: "Enforce valid defineEmits() usage (no type+runtime args, no local references, single call)",
    default_severity: Severity::Error,
};

/// Enforce valid `defineEmits` usage.
pub struct ValidDefineEmits;

impl ScriptRule for ValidDefineEmits {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        // Bindings declared at the top level of `<script setup>`. `defineEmits` is
        // hoisted above them, so its runtime argument must not reference any.
        let locals = collect_local_bindings(program);

        let mut visitor = ValidDefineEmitsVisitor {
            offset,
            result,
            locals: &locals,
            seen_call: false,
        };
        visitor.visit_program(program);
    }
}

struct ValidDefineEmitsVisitor<'locals, 'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    locals: &'locals FxHashSet<&'locals str>,
    seen_call: bool,
}

impl<'a> Visit<'a> for ValidDefineEmitsVisitor<'_, '_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if is_define_emits_call(it) {
            self.check_call(it);
        }
        walk_call_expression(self, it);
    }
}

impl ValidDefineEmitsVisitor<'_, '_> {
    fn check_call(&mut self, call: &CallExpression<'_>) {
        // (1) at most one call.
        if self.seen_call {
            self.report(
                call.span,
                "defineEmits() has already been called.",
                "Merge all emitted events into a single defineEmits() call; \
                 only one is allowed per <script setup> block.",
            );
            return;
        }
        self.seen_call = true;

        let has_type_argument = call
            .type_arguments
            .as_ref()
            .is_some_and(|arguments| !arguments.params.is_empty());
        let runtime_argument = call.arguments.first();

        // (2) not both a type argument and a runtime argument.
        if has_type_argument && runtime_argument.is_some() {
            self.report(
                call.span,
                "defineEmits() cannot accept both a type argument and a runtime argument.",
                "Declare emitted events either with the type argument \
                 (`defineEmits<{ change: [id: number] }>()`) or with a runtime argument \
                 (`defineEmits(['change'])`), not both.",
            );
            return;
        }

        // (3) the runtime argument must not reference locally-declared variables.
        if let Some(argument) = runtime_argument
            && let Some(expression) = argument.as_expression()
        {
            let mut finder = LocalReferenceFinder {
                locals: self.locals,
                found: None,
            };
            finder.visit_expression(expression);
            if let Some(span) = finder.found {
                self.report(
                    span,
                    "defineEmits() cannot reference locally-declared variables.",
                    "defineEmits() is hoisted above the component setup, so its argument \
                     can only use imports or literals, not locally-declared variables.",
                );
            }
        }
    }

    fn report(&mut self, span: Span, message: &'static str, help: &'static str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.result
            .add_diagnostic(LintDiagnostic::error(META.name, message, start, end).with_help(help));
    }
}

/// Walks an expression looking for the first identifier reference that resolves
/// to a locally-declared binding.
struct LocalReferenceFinder<'locals> {
    locals: &'locals FxHashSet<&'locals str>,
    found: Option<Span>,
}

impl<'a> Visit<'a> for LocalReferenceFinder<'_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if self.found.is_none() && self.locals.contains(it.name.as_str()) {
            self.found = Some(it.span);
        }
    }
}

/// Collect the names of bindings declared at the top level of `<script setup>`.
///
/// Only value bindings that the compiler hoists `defineEmits` above are
/// collected: `const`/`let`/`var` declarations, `function` declarations, and
/// `class` declarations. Imports and type-only declarations are intentionally
/// excluded — `defineEmits` may legitimately reference them.
fn collect_local_bindings<'a>(program: &'a Program<'a>) -> FxHashSet<&'a str> {
    let mut locals = FxHashSet::default();
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    collect_binding_names(&declarator.id, &mut locals);
                }
            }
            Statement::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    locals.insert(id.name.as_str());
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    locals.insert(id.name.as_str());
                }
            }
            _ => {}
        }
    }
    locals
}

/// Collect every identifier bound by a binding pattern (handles destructuring).
fn collect_binding_names<'a>(pattern: &BindingPattern<'a>, locals: &mut FxHashSet<&'a str>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            locals.insert(id.name.as_str());
        }
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_binding_names(&property.value, locals);
            }
            if let Some(rest) = &object.rest {
                collect_binding_names(&rest.argument, locals);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_binding_names(element, locals);
            }
            if let Some(rest) = &array.rest {
                collect_binding_names(&rest.argument, locals);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            collect_binding_names(&assignment.left, locals);
        }
    }
}

/// Whether the callee is the bare `defineEmits` compiler macro.
fn is_define_emits_call(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name.as_str() == "defineEmits"
    )
}

#[cfg(test)]
mod tests {
    use super::ValidDefineEmits;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(ValidDefineEmits));
        linter
    }

    #[test]
    fn test_valid_runtime_array() {
        let result = create_linter().lint("defineEmits(['change', 'update'])", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_type_only() {
        let result = create_linter().lint("defineEmits<{ change: [id: number] }>()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_runtime_references_import() {
        // An imported binding is not a local declaration, so referencing it is fine.
        let result =
            create_linter().lint("import { events } from './events'\ndefineEmits(events)", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_define_emits() {
        let result = create_linter().lint("const props = defineProps<{ a: number }>()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_assigned_to_binding() {
        let result = create_linter().lint("const emit = defineEmits(['change'])", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_type_and_runtime_argument() {
        let result = create_linter().lint("defineEmits<Emits>(['change'])", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_references_local_variable() {
        let result = create_linter().lint("const local = ['change']\ndefineEmits(local)", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_references_local_function() {
        let result = create_linter().lint("function make() {}\ndefineEmits({ change: make })", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_references_destructured_local() {
        let result = create_linter().lint("const { a } = obj\ndefineEmits([a])", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_duplicate_call() {
        let result = create_linter().lint("defineEmits(['change'])\ndefineEmits(['update'])", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_no_warn_nested_define_emits_in_function() {
        // Only the macro itself is checked; an identifier named `defineEmits`
        // used inside a function body is not flagged for local references.
        let result = create_linter().lint("function make() { return defineEmits(['change']) }", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_offset_applied() {
        let result = create_linter().lint("defineEmits<Emits>(['change'])", 30);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.diagnostics[0].start, 30);
    }
}
