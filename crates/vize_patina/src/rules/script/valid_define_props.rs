//! script/valid-define-props
//!
//! Enforce valid usage of the `defineProps` compiler macro in `<script setup>`.
//!
//! `defineProps` declares a component's props. It has three constraints, each
//! enforced here:
//!
//! 1. It may be called **at most once** per `<script setup>` block.
//! 2. It must **not** be given both a type argument and a runtime argument
//!    (`defineProps<{ ... }>({ ... })`): props are declared either by type *or*
//!    by runtime object, never both.
//! 3. Its runtime argument must **not** reference locally-declared variables:
//!    `defineProps` is hoisted by the compiler, so referencing module-local
//!    bindings is unsafe.
//!
//! Mirrors [`vue/valid-define-props`](https://eslint.vuejs.org/rules/valid-define-props.html).
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! defineProps<{ foo: string }>({ foo: String }) // both type and runtime args
//! const foo = 'bar'
//! defineProps({ foo })                          // references local variable
//! defineProps({ foo: String })
//! defineProps({ bar: Number })                  // duplicate call
//! ```
//!
//! ### Valid
//! ```ts
//! defineProps<{ foo: string }>()
//! defineProps({ foo: String })
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, IdentifierReference, Program, Statement,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::{GetSpan, Span};
use vize_carton::{CompactString, FxHashSet};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/valid-define-props",
    description: "Enforce valid defineProps() usage (single call, not both type and runtime args, no local references)",
    default_severity: Severity::Error,
};

/// Enforce valid `defineProps` usage.
pub struct ValidDefineProps;

impl ScriptRule for ValidDefineProps {
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
        let mut visitor = ValidDefinePropsVisitor {
            offset,
            result,
            seen_call: false,
            local_bindings: collect_top_level_bindings(program),
        };
        visitor.visit_program(program);
    }
}

struct ValidDefinePropsVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    seen_call: bool,
    local_bindings: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for ValidDefinePropsVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if is_define_props_call(it) {
            self.check_call(it);
        }
        walk_call_expression(self, it);
    }
}

impl ValidDefinePropsVisitor<'_> {
    fn check_call(&mut self, call: &CallExpression<'_>) {
        // (a) at most one call.
        if self.seen_call {
            self.report(
                call.span,
                "defineProps() has been called multiple times.",
                "Merge all props into a single defineProps() call; only one is \
                 allowed per <script setup> block.",
            );
            return;
        }
        self.seen_call = true;

        let runtime_arg = call.arguments.first();

        // (b) not both a type argument and a runtime argument.
        if call.type_arguments.is_some()
            && let Some(argument) = runtime_arg
        {
            self.report(
                argument.span(),
                "defineProps() has both a type argument and a runtime argument.",
                "Declare props with either a type argument (`defineProps<{ ... }>()`) \
                 or a runtime argument (`defineProps({ ... })`), not both.",
            );
            return;
        }

        // (c) runtime argument must not reference locally-declared variables.
        if let Some(Argument::ObjectExpression(object)) = runtime_arg {
            let hit = {
                let mut finder = LocalReferenceFinder {
                    local_bindings: &self.local_bindings,
                    offset: self.offset,
                    hit: None,
                };
                finder.visit_object_expression(object);
                finder.hit
            };
            if let Some((start, end)) = hit {
                self.report_raw(
                    start,
                    end,
                    "defineProps() is referencing locally declared variables.",
                    "Inline the value or move the declaration out of <script setup>; \
                     defineProps() is hoisted and cannot reference module-local bindings.",
                );
            }
        }
    }

    /// Report a diagnostic for a span relative to the script block (the
    /// `offset` is added to translate it into the original file).
    fn report(&mut self, span: Span, message: &'static str, help: &'static str) {
        let start = self.offset as u32 + span.start;
        let end = self.offset as u32 + span.end;
        self.report_raw(start, end, message, help);
    }

    /// Report a diagnostic for an already-absolute `start`/`end` span.
    fn report_raw(&mut self, start: u32, end: u32, message: &'static str, help: &'static str) {
        self.result
            .add_diagnostic(LintDiagnostic::error(META.name, message, start, end).with_help(help));
    }
}

/// Locates the first reference to a locally-declared binding inside a
/// `defineProps` runtime argument, recording its **absolute** span.
struct LocalReferenceFinder<'b> {
    local_bindings: &'b FxHashSet<CompactString>,
    offset: usize,
    hit: Option<(u32, u32)>,
}

impl<'a> Visit<'a> for LocalReferenceFinder<'_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        if self.hit.is_some() {
            return;
        }
        if self.local_bindings.contains(it.name.as_str()) {
            let start = self.offset as u32 + it.span.start;
            let end = self.offset as u32 + it.span.end;
            self.hit = Some((start, end));
        }
    }
}

/// Collect the names bound by top-level (module-scope) declarations: `const` /
/// `let` / `var`, function, and class declarations. These are the bindings a
/// hoisted `defineProps()` must not reference.
fn collect_top_level_bindings(program: &Program<'_>) -> FxHashSet<CompactString> {
    let mut bindings = FxHashSet::default();
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    collect_binding_names(&declarator.id, &mut bindings);
                }
            }
            Statement::FunctionDeclaration(function) => {
                if let Some(id) = &function.id {
                    bindings.insert(CompactString::new(id.name.as_str()));
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    bindings.insert(CompactString::new(id.name.as_str()));
                }
            }
            _ => {}
        }
    }
    bindings
}

/// Add every identifier bound by a binding pattern (including destructured
/// bindings) to `bindings`.
fn collect_binding_names(pattern: &BindingPattern<'_>, bindings: &mut FxHashSet<CompactString>) {
    for id in pattern.get_binding_identifiers() {
        bindings.insert(CompactString::new(id.name.as_str()));
    }
}

/// Whether the callee is the bare `defineProps` compiler macro.
fn is_define_props_call(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(identifier) if identifier.name.as_str() == "defineProps"
    )
}

#[cfg(test)]
mod tests {
    use super::ValidDefineProps;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(ValidDefineProps));
        linter
    }

    #[test]
    fn test_valid_type_only() {
        let result = create_linter().lint("defineProps<{ foo: string }>()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_runtime_only() {
        let result = create_linter().lint("defineProps({ foo: String })", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_runtime_array() {
        let result = create_linter().lint("defineProps(['foo', 'bar'])", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_empty_runtime() {
        let result = create_linter().lint("defineProps({})", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_no_define_props() {
        let result = create_linter().lint("const x = defineEmits<{ a: [] }>()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_references_imported_type_constructor() {
        // `String`/`Number` are globals, not locally declared.
        let result = create_linter().lint("defineProps({ foo: String, bar: Number })", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_assigned_to_binding() {
        let result = create_linter().lint("const props = defineProps<{ foo: string }>()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_both_type_and_runtime() {
        let result = create_linter().lint("defineProps<{ foo: string }>({ foo: String })", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_references_local_variable() {
        let result = create_linter().lint("const foo = { type: String }\ndefineProps({ foo })", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_references_local_variable_as_value() {
        let result = create_linter().lint("const def = String\ndefineProps({ foo: def })", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_references_local_function() {
        let result = create_linter().lint(
            "function validate() {}\ndefineProps({ foo: { validator: validate } })",
            0,
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_duplicate_call() {
        let result = create_linter().lint(
            "defineProps({ foo: String })\ndefineProps({ bar: Number })",
            0,
        );
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_duplicate_call_type_args() {
        let result = create_linter().lint(
            "defineProps<{ foo: string }>()\ndefineProps<{ bar: number }>()",
            0,
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_both_type_and_runtime_takes_priority_over_local_ref() {
        // The both-args error short-circuits; only one diagnostic is produced.
        let result = create_linter().lint(
            "const foo = String\ndefineProps<{ foo: string }>({ foo })",
            0,
        );
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_offset_applied() {
        let result = create_linter().lint("defineProps<{ foo: string }>({ foo: String })", 30);
        assert_eq!(result.error_count, 1);
        assert!(result.diagnostics[0].start >= 30);
    }
}
