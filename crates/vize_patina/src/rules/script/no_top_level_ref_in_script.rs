//! script/no-top-level-ref-in-script
//!
//! Disallow top-level ref/reactive in non-setup scripts to prevent Cross-Request State Pollution.
//!
//! In SSR (Server-Side Rendering) scenarios, top-level reactive state in regular
//! `<script>` blocks (not `<script setup>`) is shared across all requests.
//! This can lead to data leaking between different users' requests.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script>
//! // This state is shared across all requests in SSR!
//! const count = ref(0)
//! const user = reactive({ name: '' })
//!
//! export default {
//!   setup() {
//!     return { count, user }
//!   }
//! }
//! </script>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup>
//! // Script setup creates fresh state per request
//! const count = ref(0)
//! </script>
//!
//! <script>
//! // Constants are fine
//! const API_URL = 'https://api.example.com'
//!
//! // Functions that create state are fine
//! function createState() {
//!   return reactive({ count: 0 })
//! }
//!
//! export default {
//!   setup() {
//!     // Create state inside setup
//!     const count = ref(0)
//!     return { count }
//!   }
//! }
//! </script>
//! ```

use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    ArrowFunctionExpression, CallExpression, Expression, Function, Program, Statement,
    VariableDeclarationKind,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_span::Span;
use oxc_syntax::scope::ScopeFlags;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-top-level-ref-in-script",
    description: "Disallow top-level ref/reactive to prevent Cross-Request State Pollution",
    default_severity: Severity::Error,
};

/// Prevent top-level reactive state in non-setup scripts
pub struct NoTopLevelRefInScript;

impl ScriptRule for NoTopLevelRefInScript {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    // Top-level state in `<script setup>` is fresh per component instance, so it
    // is the idiomatic pattern there and must not be flagged. Only plain
    // `<script>` (module scope) leaks reactive state across SSR requests.
    fn runs_on_script_setup(&self) -> bool {
        false
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
        for statement in &program.body {
            let Statement::VariableDeclaration(declaration) = statement else {
                continue;
            };
            if !matches!(
                declaration.kind,
                VariableDeclarationKind::Const | VariableDeclarationKind::Let
            ) {
                continue;
            }

            for declarator in &declaration.declarations {
                if let Some(init) = &declarator.init {
                    let mut visitor = TopLevelRefVisitor { offset, result };
                    visitor.visit_expression(init);
                }
            }
        }
    }
}

struct TopLevelRefVisitor<'result> {
    offset: usize,
    result: &'result mut ScriptLintResult,
}

impl<'a> Visit<'a> for TopLevelRefVisitor<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Some(span) = reactive_callee_span(it) {
            let start = self.offset as u32 + span.start;
            let end = self.offset as u32 + span.end;
            self.result.add_diagnostic(
                LintDiagnostic::error(
                    META.name,
                    "Top-level reactive state in <script> can cause Cross-Request State Pollution in SSR",
                    start,
                    end,
                )
                .with_help(
                    "Move reactive state inside setup() or use <script setup>. \
                     Top-level state is shared across requests in SSR.",
                ),
            );
        }

        walk_call_expression(self, it);
    }

    fn visit_arrow_function_expression(&mut self, _it: &ArrowFunctionExpression<'a>) {}

    fn visit_function(&mut self, _it: &Function<'a>, _flags: ScopeFlags) {}
}

fn reactive_callee_span(call: &CallExpression<'_>) -> Option<Span> {
    let Expression::Identifier(identifier) = &call.callee else {
        return None;
    };
    if matches!(
        identifier.name.as_str(),
        "ref" | "reactive" | "computed" | "shallowRef" | "shallowReactive"
    ) {
        Some(Span::new(identifier.span.start, identifier.span.end + 1))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::NoTopLevelRefInScript;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoTopLevelRefInScript));
        linter
    }

    #[test]
    fn test_valid_inside_setup() {
        let linter = create_linter();
        let result = linter.lint(
            r#"export default {
  setup() {
    const count = ref(0)
    return { count }
  }
}"#,
            0,
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_inside_function() {
        let linter = create_linter();
        let result = linter.lint(
            r#"function createState() {
  return reactive({ count: 0 })
}"#,
            0,
        );
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_top_level_ref() {
        let linter = create_linter();
        let result = linter.lint("const count = ref(0)", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_top_level_reactive() {
        let linter = create_linter();
        let result = linter.lint("const state = reactive({ count: 0 })", 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_const_string() {
        let linter = create_linter();
        let result = linter.lint("const API_URL = 'https://api.example.com'", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_arrow_function_factory() {
        let linter = create_linter();
        let result = linter.lint("const createState = () => ref(0)", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_ignores_string_literal_source() {
        let linter = create_linter();
        let result = linter.lint(r#"const source = "const count = ref(0)""#, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_nested_initializer_ref() {
        let linter = create_linter();
        let result = linter.lint("const state = { count: ref(0) }", 0);
        assert_eq!(result.error_count, 1);
    }
}
