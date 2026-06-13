//! script/require-typed-ref
//!
//! Require an explicit type argument on a `ref()` whose initial value cannot be
//! inferred.
//!
//! When `ref()` is called with no argument, or with `null` / `undefined`, the
//! inferred type collapses to `Ref<undefined>` / `Ref<null>` (or, in loose
//! configs, `Ref<any>`), which silently defeats type-checking on every later
//! read and write. Such a ref should carry an explicit type argument
//! (`ref<string>()`, `ref<User | null>(null)`) so the intended element type is
//! known up front.
//!
//! Only the un-inferable initializers are flagged — `ref()`, `ref(null)`, and
//! `ref(undefined)` *without* a type argument. A `ref(0)` (inferable from the
//! literal) or any `ref<T>(...)` (already typed) is left alone. The `ref` must be
//! the one imported from `vue` (directly or under an alias).
//!
//! Mirrors [`vue/require-typed-ref`](https://eslint.vuejs.org/rules/require-typed-ref.html),
//! which applies to TypeScript only.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! import { ref } from 'vue'
//!
//! const a = ref()           // Ref<undefined>
//! const b = ref(null)       // Ref<null>
//! const c = ref(undefined)  // Ref<undefined>
//! ```
//!
//! ### Valid
//! ```ts
//! import { ref } from 'vue'
//!
//! const a = ref<string>()
//! const b = ref<User | null>(null)
//! const c = ref(0)          // inferred Ref<number>
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    CallExpression, Expression, ImportDeclaration, ImportDeclarationSpecifier, Program,
};
use oxc_ast_visit::{
    Visit,
    walk::{walk_call_expression, walk_import_declaration},
};
use vize_carton::{CompactString, FxHashSet};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/require-typed-ref",
    description: "Require an explicit type argument on a ref() initialized with no value, null, or undefined",
    default_severity: Severity::Warning,
};

const MESSAGE: &str = "This ref() should have an explicit type argument.";
const HELP: &str = "A ref() initialized with no value, `null`, or `undefined` infers `Ref<undefined>`/`Ref<null>`, \
     which disables type-checking. Add a type argument, e.g. `ref<string>()`.";

/// Require an explicit type argument on un-inferable `ref()` calls.
pub struct RequireTypedRef;

impl ScriptRule for RequireTypedRef {
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
        let mut visitor = RequireTypedRefVisitor {
            offset,
            result,
            ref_aliases: FxHashSet::default(),
            ref_imported: false,
        };
        visitor.visit_program(program);
    }
}

struct RequireTypedRefVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
    /// Local names bound to the `ref` import from `vue` (handles `ref as r`).
    ref_aliases: FxHashSet<CompactString>,
    /// Whether `ref` was imported from `vue` at all. Only then do bare `ref(...)`
    /// calls refer to the reactivity helper rather than some unrelated function.
    ref_imported: bool,
}

impl<'a> Visit<'a> for RequireTypedRefVisitor<'_> {
    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        if it.source.value.as_str() == "vue"
            && let Some(specifiers) = &it.specifiers
        {
            for specifier in specifiers {
                let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                    continue;
                };
                if specifier.imported.name().as_str() == "ref" {
                    self.ref_imported = true;
                    self.ref_aliases
                        .insert(CompactString::new(specifier.local.name.as_str()));
                }
            }
        }
        walk_import_declaration(self, it);
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if self.is_ref_call(it) && it.type_arguments.is_none() && init_is_uninferable(it) {
            let start = self.offset as u32 + it.span.start;
            let end = self.offset as u32 + it.span.end;
            self.result.add_diagnostic(
                LintDiagnostic::warn(META.name, MESSAGE, start, end)
                    .with_label("missing type argument", start, end)
                    .with_help(HELP),
            );
        }
        walk_call_expression(self, it);
    }
}

impl RequireTypedRefVisitor<'_> {
    /// Whether the callee is the `ref` helper imported from `vue`. Requires the
    /// import to be present so a local `ref` in a non-vue file is not flagged.
    fn is_ref_call(&self, call: &CallExpression<'_>) -> bool {
        if !self.ref_imported {
            return false;
        }
        matches!(
            &call.callee,
            Expression::Identifier(identifier) if self.ref_aliases.contains(identifier.name.as_str())
        )
    }
}

/// Whether the call's initializer is un-inferable: no argument, or a single
/// `null` / `undefined` argument.
fn init_is_uninferable(call: &CallExpression<'_>) -> bool {
    match call.arguments.first() {
        None => true,
        Some(first) if call.arguments.len() == 1 => match first.as_expression() {
            Some(expression) => is_null_or_undefined(expression),
            None => false,
        },
        _ => false,
    }
}

/// Whether an expression is the literal `null` or the `undefined` identifier.
fn is_null_or_undefined(expression: &Expression<'_>) -> bool {
    match expression {
        Expression::NullLiteral(_) => true,
        Expression::Identifier(identifier) => identifier.name.as_str() == "undefined",
        Expression::ParenthesizedExpression(paren) => is_null_or_undefined(&paren.expression),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::RequireTypedRef;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(RequireTypedRef));
        linter
    }

    const IMPORT: &str = "import { ref } from 'vue'\n";

    #[test]
    fn test_valid_typed_empty() {
        let result = create_linter().lint(&format!("{IMPORT}const a = ref<string>()"), 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_typed_null() {
        let result = create_linter().lint(&format!("{IMPORT}const a = ref<User | null>(null)"), 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_inferable_literal() {
        let result = create_linter().lint(&format!("{IMPORT}const a = ref(0)"), 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_inferable_object() {
        let result = create_linter().lint(&format!("{IMPORT}const a = ref({{ count: 0 }})"), 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_ref_not_imported_from_vue() {
        // `ref` is not imported from vue, so this is some other function.
        let result = create_linter().lint("const a = ref()", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_bare_ref() {
        let result = create_linter().lint(&format!("{IMPORT}const a = ref()"), 0);
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_ref_null() {
        let result = create_linter().lint(&format!("{IMPORT}const a = ref(null)"), 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_ref_undefined() {
        let result = create_linter().lint(&format!("{IMPORT}const a = ref(undefined)"), 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_aliased_ref() {
        let result = create_linter().lint("import { ref as r } from 'vue'\nconst a = r()", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_typed_undefined() {
        let result = create_linter().lint(&format!("{IMPORT}const a = ref<string>(undefined)"), 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_offset_applied() {
        let source = format!("{IMPORT}const a = ref()");
        let result = create_linter().lint(&source, 100);
        assert_eq!(result.warning_count, 1);
        let call_start = source.find("ref()").unwrap() as u32 + 100;
        assert_eq!(result.diagnostics[0].start, call_start);
    }
}
