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
//! Type-only exports (`export type`, `export interface`, ambient `export
//! declare`, all-type specifier lists, and `export type * from`) are erased at
//! compile time and never reach the compiled `setup()`, so they stay allowed —
//! matching what `@vue/compiler-sfc` accepts.
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
//! <script setup lang="ts">
//! export type Props = { count: number } // type-only, erased at compile time
//! const props = defineProps<Props>()
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
use oxc_ast::ast::{Declaration, ExportNamedDeclaration, Program, Statement};
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

/// What kind of runtime top-level export a statement is, if any. Type-only
/// exports return `None`: they are erased at compile time and legal in
/// `<script setup>`.
fn export_statement(statement: &Statement<'_>) -> Option<(Span, &'static str)> {
    match statement {
        Statement::ExportNamedDeclaration(export) => {
            if is_type_only_named_export(export) {
                None
            } else {
                Some((export.span, "named export"))
            }
        }
        Statement::ExportDefaultDeclaration(export) => Some((export.span, "default export")),
        Statement::ExportAllDeclaration(export) => {
            if export.export_kind.is_type() {
                None
            } else {
                Some((export.span, "re-export"))
            }
        }
        _ => None,
    }
}

/// Whether a named export lives entirely in type space.
///
/// `@vue/compiler-sfc` only rejects exports that would have to survive into
/// the compiled `setup()` function. `export type ...`, `export interface`,
/// ambient `export declare ...`, and specifier lists where every specifier is
/// `type` are all erased by the TypeScript compiler, so they never produce a
/// runtime export. Enums are excluded: they emit a runtime object.
fn is_type_only_named_export(export: &ExportNamedDeclaration<'_>) -> bool {
    if export.export_kind.is_type() {
        return true;
    }
    match &export.declaration {
        Some(Declaration::TSTypeAliasDeclaration(_) | Declaration::TSInterfaceDeclaration(_)) => {
            true
        }
        Some(Declaration::VariableDeclaration(declaration)) => declaration.declare,
        Some(Declaration::FunctionDeclaration(declaration)) => declaration.declare,
        Some(Declaration::ClassDeclaration(declaration)) => declaration.declare,
        Some(Declaration::TSEnumDeclaration(declaration)) => declaration.declare,
        Some(Declaration::TSModuleDeclaration(declaration)) => declaration.declare,
        Some(_) => false,
        None => {
            !export.specifiers.is_empty()
                && export
                    .specifiers
                    .iter()
                    .all(|specifier| specifier.export_kind.is_type())
        }
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
mod tests;
