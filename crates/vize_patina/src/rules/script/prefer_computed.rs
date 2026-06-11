//! script/prefer-computed
//!
//! Prefer computed properties over manually syncing reactive state.
//!
//! When a value can be derived from other reactive state, use `computed()`
//! instead of manually updating a separate `ref` with a watcher.
//!
//! This follows the principle: "reactive state that can be computed from
//! other state should use computed properties, not be actively defined."
//!
//! ## Examples
//!
//! ### Not Recommended
//! ```ts
//! const count = ref(0)
//! const doubled = ref(0)
//!
//! watch(count, (val) => {
//!   doubled.value = val * 2
//! })
//! ```
//!
//! ### Recommended
//! ```ts
//! const count = ref(0)
//! const doubled = computed(() => count.value * 2)
//! ```

use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, CallExpression, Expression, ImportDeclaration,
    ImportDeclarationSpecifier, Program,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_assignment_expression, walk_call_expression, walk_import_declaration},
};
use oxc_span::Span;
use oxc_syntax::operator::AssignmentOperator;
use vize_carton::{CompactString, FxHashSet};

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/prefer-computed",
    description: "Prefer computed() for derived reactive state",
    default_severity: Severity::Warning,
};

/// Prefer computed over watched refs
pub struct PreferComputed;

impl ScriptRule for PreferComputed {
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
        let mut visitor = PreferComputedVisitor {
            offset,
            result,
            watch_aliases: FxHashSet::default(),
        };
        visitor.visit_program(program);
    }
}

struct PreferComputedVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    /// Local names bound to Vue's `watch` import (covers `watch as observe`).
    watch_aliases: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for PreferComputedVisitor<'_> {
    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        if it.source.value.as_str() == "vue"
            && let Some(specifiers) = &it.specifiers
        {
            for specifier in specifiers {
                let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                    continue;
                };
                if specifier.imported.name().as_str() == "watch" {
                    self.watch_aliases
                        .insert(CompactString::new(specifier.local.name.as_str()));
                }
            }
        }

        walk_import_declaration(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(callee_span) = self.watch_callee_span(it)
            && let Some(callback) = watch_callback(it)
            && callback_assigns_ref_value(callback)
        {
            let start = self.offset as u32 + callee_span.start;
            let end = self.offset as u32 + callee_span.end;
            self.result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "Consider using computed() instead of watch() for derived state",
                    start,
                    end,
                )
                .with_help(
                    "If the watch callback only assigns to a ref based on the watched value, \
                     use computed() instead: `const derived = computed(() => source.value * 2)`",
                ),
            );
        }

        walk_call_expression(self, it);
    }
}

impl PreferComputedVisitor<'_> {
    /// Span of the callee when this is a `watch(...)` call.
    ///
    /// Matches the bare `watch` identifier (Nuxt-style auto-imports) and any
    /// local alias of Vue's `watch` import (`import { watch as observe }`).
    fn watch_callee_span(&self, call: &CallExpression<'_>) -> Option<Span> {
        let Expression::Identifier(identifier) = &call.callee else {
            return None;
        };
        let name = identifier.name.as_str();
        if name == "watch" || self.watch_aliases.contains(name) {
            Some(identifier.span)
        } else {
            None
        }
    }
}

/// The inline callback of a `watch(source, callback, options?)` call, if any.
fn watch_callback<'a, 'b>(call: &'b CallExpression<'a>) -> Option<&'b Expression<'a>> {
    let callback = call.arguments.get(1)?.as_expression()?;
    Some(unwrap_expression(callback))
}

/// Strip parentheses and TS-only wrappers so the underlying callback is seen.
fn unwrap_expression<'a, 'b>(expression: &'b Expression<'a>) -> &'b Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(paren) => unwrap_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => unwrap_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            unwrap_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => unwrap_expression(&ts_non_null.expression),
        _ => expression,
    }
}

/// Whether the watch callback contains a plain `<target>.value = ...`
/// assignment, i.e. it manually syncs derived reactive state.
fn callback_assigns_ref_value(callback: &Expression<'_>) -> bool {
    let mut finder = RefValueAssignmentFinder { found: false };
    match callback {
        Expression::ArrowFunctionExpression(arrow) => {
            finder.visit_function_body(&arrow.body);
        }
        Expression::FunctionExpression(function) => {
            if let Some(body) = &function.body {
                finder.visit_function_body(body);
            }
        }
        _ => return false,
    }
    finder.found
}

struct RefValueAssignmentFinder {
    found: bool,
}

impl<'a> Visit<'a> for RefValueAssignmentFinder {
    fn visit_assignment_expression(&mut self, it: &AssignmentExpression<'a>) {
        if self.found {
            return;
        }
        if it.operator == AssignmentOperator::Assign
            && let AssignmentTarget::StaticMemberExpression(member) = &it.left
            && member.property.name == "value"
        {
            self.found = true;
            return;
        }
        walk_assignment_expression(self, it);
    }
}

#[cfg(test)]
mod tests {
    use super::PreferComputed;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(PreferComputed));
        linter
    }

    #[test]
    fn test_warn_watch_with_value_assignment() {
        let linter = create_linter();
        let result = linter.lint(
            r#"
const count = ref(0)
const doubled = ref(0)
watch(count, (val) => {
  doubled.value = val * 2
})
"#,
            0,
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_valid_watch_without_value_assignment() {
        let linter = create_linter();
        let result = linter.lint(
            r#"
const count = ref(0)
watch(count, (val) => {
  console.log('count changed:', val)
})
"#,
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_no_watch() {
        let linter = create_linter();
        let result = linter.lint(
            r#"
const count = ref(0)
const doubled = computed(() => count.value * 2)
"#,
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_no_warn_pattern_inside_string_literal() {
        // The old byte scanner flagged `watch(` + `.value =` even inside a
        // string literal. The AST check must not.
        let linter = create_linter();
        let result = linter.lint(
            r#"
const example = "watch(count, (val) => { doubled.value = val * 2 })"
console.log(example)
"#,
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_no_warn_pattern_inside_comment() {
        // The old byte scanner flagged the pattern inside comments.
        let linter = create_linter();
        let result = linter.lint(
            r#"
// watch(count, (val) => { doubled.value = val * 2 })
const doubled = computed(() => count.value * 2)
"#,
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_no_warn_value_comparison_in_callback() {
        // `.value ===` contains the `.value =` substring the old scanner
        // matched; a comparison is not derived-state syncing.
        let linter = create_linter();
        let result = linter.lint(
            r#"
watch(count, (val) => {
  if (doubled.value === val) {
    console.log(val)
  }
})
"#,
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_no_warn_value_assignment_after_watch_call() {
        // The old scanner matched any `.value =` within 200 bytes after
        // `watch(`, even outside the callback.
        let linter = create_linter();
        let result = linter.lint(
            r#"
watch(count, (val) => {
  console.log(val)
})
doubled.value = 5
"#,
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warn_aliased_watch_import() {
        // `import { watch as observe }` never produced the `watch(` byte
        // pattern, so the old scanner missed it entirely.
        let linter = create_linter();
        let result = linter.lint(
            r#"
import { watch as observe } from 'vue'
const doubled = ref(0)
observe(count, (val) => {
  doubled.value = val * 2
})
"#,
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_warn_watch_with_space_before_paren() {
        // `watch (source, cb)` passed the old fast bailout but the finder only
        // searched for `watch(`, so it was silently missed.
        let linter = create_linter();
        let result = linter.lint(
            r#"
watch (count, (val) => {
  doubled.value = val * 2
})
"#,
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_warn_assignment_beyond_200_bytes() {
        // The old scanner gave up when `.value =` sat more than 200 bytes
        // after `watch(`; long callbacks were missed.
        let linter = create_linter();
        let source = r#"
watch(count, (val) => {
  console.log('step 0 of a fairly long watch callback body, padding away')
  console.log('step 1 of a fairly long watch callback body, padding away')
  console.log('step 2 of a fairly long watch callback body, padding away')
  console.log('step 3 of a fairly long watch callback body, padding away')
  console.log('step 4 of a fairly long watch callback body, padding away')
  console.log('step 5 of a fairly long watch callback body, padding away')
  doubled.value = val * 2
})
"#;
        assert!(source.find(".value =").unwrap() > 200);
        let result = linter.lint(source, 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_warn_function_expression_callback() {
        let linter = create_linter();
        let result = linter.lint(
            r#"
watch(count, function (val) {
  doubled.value = val * 2
})
"#,
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_no_warn_unrelated_function_ending_in_watch() {
        // Identifiers merely ending in `watch` (e.g. `unwatch(...)`) matched
        // the old substring finder.
        let linter = create_linter();
        let result = linter.lint(
            r#"
unwatch(count, (val) => {
  doubled.value = val * 2
})
"#,
            0,
        );
        assert_eq!(result.warning_count, 0);
    }
}
