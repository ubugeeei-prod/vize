//! script/no-export-in-script-setup
//!
//! Disallow `export` statements inside `<script setup>`.
//!
//! A `<script setup>` block is compiled into the component's `setup()` function:
//! every top-level binding is automatically exposed to the template, and the
//! block itself is not a real ES module. A top-level `export` there is therefore
//! meaningless and the Vue SFC compiler rejects it. To expose bindings to a
//! parent component use [`defineExpose()`](https://vuejs.org/api/sfc-script-setup.html#defineexpose).
//!
//! A normal `<script>` block legitimately uses `export default { ... }`, so the
//! rule first confirms the block is a `<script setup>` by the presence of a
//! compiler macro (`defineProps`, `defineEmits`, `defineExpose`, `defineOptions`,
//! `defineSlots`, `defineModel`, `withDefaults`) or a top-level `await`, both of
//! which are only valid inside `<script setup>`.
//!
//! ## Invalid
//! ```vue
//! <script setup>
//! const props = defineProps<{ count: number }>()
//! export const helper = () => props.count * 2 // meaningless in setup
//! export default {} // also invalid
//! </script>
//! ```
//!
//! ## Valid
//! ```vue
//! <script setup>
//! const props = defineProps<{ count: number }>()
//! const helper = () => props.count * 2
//! defineExpose({ helper })
//! </script>
//!
//! <script>
//! // A normal <script> block may export the component options.
//! export default { name: 'MyComponent' }
//! </script>
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Program, Statement};
use oxc_span::Span;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-export-in-script-setup",
    description: "Disallow export statements inside <script setup>",
    default_severity: Severity::Error,
};

/// Compiler macros that are only valid inside `<script setup>`. Their presence
/// is a reliable marker that a script block is a `<script setup>` rather than a
/// normal `<script>` (which legitimately exports its component options).
const SCRIPT_SETUP_MACROS: &[&str] = &[
    "defineProps",
    "defineEmits",
    "defineExpose",
    "defineOptions",
    "defineSlots",
    "defineModel",
    "withDefaults",
];

/// Disallow top-level `export` declarations inside `<script setup>`.
pub struct NoExportInScriptSetup;

impl ScriptRule for NoExportInScriptSetup {
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
        // A normal <script> uses `export default { ... }` for its options, so
        // only flag exports once we are confident this block is a <script setup>.
        if !is_script_setup_block(program, source) {
            return;
        }

        for statement in &program.body {
            if let Some((span, kind)) = export_statement(statement) {
                report(kind, span, offset, result);
            }
        }
    }
}

/// What kind of top-level export a statement is, if any.
fn export_statement(statement: &Statement<'_>) -> Option<(Span, &'static str)> {
    match statement {
        Statement::ExportNamedDeclaration(export) => Some((export.span, "named export")),
        Statement::ExportDefaultDeclaration(export) => Some((export.span, "default export")),
        Statement::ExportAllDeclaration(export) => Some((export.span, "re-export")),
        _ => None,
    }
}

fn report(kind: &str, span: Span, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(
        LintDiagnostic::error(
            META.name,
            "Unexpected `export` in `<script setup>`: it is compiled into setup() and the export is meaningless",
            start,
            end,
        )
        .with_label(kind, start, end)
        .with_help(
            "Remove the `export`. `<script setup>` exposes top-level bindings to the \
             template automatically; use `defineExpose()` to expose bindings to a parent.",
        ),
    );
}

/// Whether the parsed block is a `<script setup>`.
///
/// The script-rule trait does not tell a rule which SFC block it is checking, so
/// this distinguishes a `<script setup>` from a normal `<script>` by a feature
/// that is only legal inside `<script setup>`: a compiler-macro call or a
/// top-level `await`. A normal `<script>` has neither, so its `export default`
/// component options are never flagged.
fn is_script_setup_block(program: &Program<'_>, source: &str) -> bool {
    program_has_top_level_await(program) || source_uses_script_setup_macro(source)
}

/// Whether the block uses a compiler macro that is exclusive to `<script setup>`.
///
/// A byte-level prefilter mirroring the convention used by other script rules
/// (e.g. `no-with-defaults`). The macros are not valid identifiers to import in
/// a normal `<script>`, so a textual occurrence is a strong setup signal.
fn source_uses_script_setup_macro(source: &str) -> bool {
    let bytes = source.as_bytes();
    SCRIPT_SETUP_MACROS
        .iter()
        .any(|macro_name| memchr::memmem::find(bytes, macro_name.as_bytes()).is_some())
}

/// Whether the program contains a top-level `await`, which is only valid inside
/// `<script setup>` (a normal `<script>` is not an async context).
fn program_has_top_level_await(program: &Program<'_>) -> bool {
    program
        .body
        .iter()
        .any(|statement| statement_has_top_level_await(statement))
}

fn statement_has_top_level_await(statement: &Statement<'_>) -> bool {
    use oxc_ast::ast::Expression;

    // Only the directly-awaited forms that can appear as a top-level statement
    // are needed here; awaits nested inside functions are not "top level".
    match statement {
        Statement::ExpressionStatement(stmt) => {
            matches!(&stmt.expression, Expression::AwaitExpression(_))
        }
        Statement::VariableDeclaration(decl) => decl.declarations.iter().any(|declarator| {
            matches!(
                declarator.init.as_ref(),
                Some(Expression::AwaitExpression(_))
            )
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::NoExportInScriptSetup;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoExportInScriptSetup));
        linter
    }

    // --- Invalid: exports inside a recognizable <script setup> ---

    #[test]
    fn test_invalid_named_export_with_macro() {
        let source = r#"
const props = defineProps<{ count: number }>()
export const helper = () => props.count
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_default_export_with_macro() {
        let source = r#"
defineProps<{ count: number }>()
export default {}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_export_all_with_macro() {
        let source = r#"
defineEmits(['change'])
export * from './helpers'
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_export_named_specifier_with_macro() {
        let source = r#"
defineExpose({})
const a = 1
export { a }
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_multiple_exports_all_reported() {
        let source = r#"
const props = defineProps<{ count: number }>()
export const a = 1
export function b() {}
export default {}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 3);
    }

    #[test]
    fn test_invalid_export_detected_via_top_level_await() {
        // No compiler macro, but a top-level await proves this is <script setup>.
        let source = r#"
const data = await fetch('/api')
export const cached = data
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_export_type_with_macro() {
        // `export type` is still a top-level ExportNamedDeclaration.
        let source = r#"
defineProps<{ count: number }>()
export type Foo = { a: number }
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 1);
    }

    // --- Valid: normal <script> (no setup markers) is never flagged ---

    #[test]
    fn test_valid_export_default_in_normal_script() {
        // A normal <script> exports its component options; not a <script setup>.
        let source = r#"
export default {
  name: 'MyComponent',
  data() {
    return { count: 0 }
  }
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_named_export_in_normal_script() {
        let source = r#"
export const API_URL = 'https://example.com'
export function helper() {
  return 1
}
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_export_all_in_normal_script() {
        let source = "export * from './helpers'\n";
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    // --- Valid: <script setup> without any export ---

    #[test]
    fn test_valid_script_setup_no_export() {
        let source = r#"
import { ref } from 'vue'
const props = defineProps<{ count: number }>()
const doubled = ref(props.count * 2)
defineExpose({ doubled })
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_import_is_not_export() {
        // Imports are allowed in <script setup>; only exports are flagged.
        let source = r#"
import Foo from './Foo.vue'
import { bar } from './bar'
const props = defineProps<{ count: number }>()
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_macro_substring_in_string_no_export() {
        // The macro byte-prefilter may trip on a string literal, but with no
        // actual export there is nothing to report.
        let source = r#"
const label = 'defineProps demo'
const x = 1
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_nested_export_keyword_not_top_level() {
        // `export` only inside a string is not a real export statement.
        let source = r#"
defineProps<{ count: number }>()
const code = 'export default {}'
"#;
        let result = create_linter().lint(source, 0);
        assert_eq!(result.error_count, 0);
    }
}
