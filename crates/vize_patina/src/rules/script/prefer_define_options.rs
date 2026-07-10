//! script/prefer-define-options
//!
//! Prefer the `defineOptions()` macro over a separate plain `<script>` block that
//! only declares `name` / `inheritAttrs`.
//!
//! With `<script setup>`, component options that have no dedicated macro (`name`,
//! `inheritAttrs`) used to require an extra plain `<script>` block:
//!
//! ```vue
//! <script>
//! export default { name: 'MyComponent', inheritAttrs: false }
//! </script>
//! <script setup>
//! // ...
//! </script>
//! ```
//!
//! Since Vue 3.3 this is expressible inline with `defineOptions({ name: '...',
//! inheritAttrs: false })`, collapsing the two blocks into one. This rule flags
//! the plain-script form when its default export is an Options object that
//! declares **only** `name` and/or `inheritAttrs` (and nothing else), suggesting
//! the move to `defineOptions`.
//!
//! The check is deliberately conservative: any other option key, any additional
//! statement, or a non-object default export means the plain `<script>` is
//! carrying real logic and is left untouched.
//!
//! ## Examples
//!
//! ### Invalid (plain `<script>` carrying only name/inheritAttrs)
//! ```ts
//! export default { name: 'MyComponent', inheritAttrs: false }
//! ```
//!
//! ### Valid
//! ```ts
//! // Real options logic — keep the plain script.
//! export default {
//!   name: 'MyComponent',
//!   data() { return { count: 0 } },
//! }
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{
    ExportDefaultDeclarationKind, ObjectExpression, ObjectPropertyKind, Program, PropertyKey,
    Statement,
};
use oxc_span::Span;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/prefer-define-options",
    description: "Prefer defineOptions() over a plain <script> that only sets name/inheritAttrs",
    default_severity: Severity::Warning,
};

const MESSAGE: &str =
    "Prefer defineOptions() over a separate <script> block that only sets name/inheritAttrs.";
const HELP: &str = "Move these options into a `defineOptions({ ... })` call in <script setup> and \
     remove this plain <script> block.";

/// Options keys that `defineOptions` covers and that, on their own, justify the
/// suggestion. If the default export declares anything else, the plain script is
/// carrying real logic and is left alone.
const ALLOWED_KEYS: [&str; 2] = ["name", "inheritAttrs"];

/// Prefer `defineOptions()` over a name/inheritAttrs-only plain `<script>`.
pub struct PreferDefineOptions;

impl ScriptRule for PreferDefineOptions {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    // This targets the *plain* `<script>` block that pairs with a `<script
    // setup>`; the options-only export never appears in `<script setup>` itself.
    fn runs_on_script_setup(&self) -> bool {
        false
    }

    #[inline]
    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        let Some((object, span)) = options_only_default_export(program) else {
            return;
        };
        if !object_has_only_allowed_keys(object) {
            return;
        }
        let start = offset as u32 + span.start;
        let end = offset as u32 + span.end;
        result.add_diagnostic(
            LintDiagnostic::warn(META.name, MESSAGE, start, end)
                .with_label("only sets name/inheritAttrs", start, end)
                .with_help(HELP),
        );
    }
}

/// The default-export object expression together with its span, but only when it
/// is the *sole* meaningful statement in the program (imports are permitted; any
/// other runtime statement disqualifies the heuristic).
fn options_only_default_export<'a>(
    program: &'a Program<'a>,
) -> Option<(&'a ObjectExpression<'a>, Span)> {
    let mut export_object: Option<(&'a ObjectExpression<'a>, Span)> = None;

    for statement in &program.body {
        match statement {
            // Imports are layout, not logic; they do not block the suggestion.
            Statement::ImportDeclaration(_) => continue,
            Statement::ExportDefaultDeclaration(export) => {
                if export_object.is_some() {
                    // More than one default export is malformed; bail out.
                    return None;
                }
                let ExportDefaultDeclarationKind::ObjectExpression(object) = &export.declaration
                else {
                    // A non-object default export (defineComponent(...), an
                    // identifier, a function, ...) carries logic; leave it.
                    return None;
                };
                export_object = Some((object, object.span));
            }
            // Any other top-level statement (a const, a function, a side-effect
            // call) means the plain script is doing real work.
            _ => return None,
        }
    }

    export_object
}

/// Whether the object declares at least one option and *every* declared option
/// is one of [`ALLOWED_KEYS`]. A spread, a computed key, or any other key makes
/// this `false` so logic-bearing option objects are never flagged.
fn object_has_only_allowed_keys(object: &ObjectExpression<'_>) -> bool {
    if object.properties.is_empty() {
        return false;
    }
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            // A spread element (`...base`) pulls in unknown options.
            return false;
        };
        if property.computed {
            return false;
        }
        let Some(name) = property_key_name(&property.key) else {
            return false;
        };
        if !ALLOWED_KEYS.contains(&name) {
            return false;
        }
    }
    true
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::PreferDefineOptions;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(PreferDefineOptions));
        linter
    }

    #[test]
    fn test_invalid_name_and_inherit_attrs() {
        let result = create_linter().lint(
            "export default { name: 'MyComponent', inheritAttrs: false }",
            0,
        );
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_name_only() {
        let result = create_linter().lint("export default { name: 'Foo' }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_inherit_attrs_only() {
        let result = create_linter().lint("export default { inheritAttrs: false }", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_with_imports_present() {
        let result = create_linter().lint(
            "import { foo } from './foo'\nexport default { name: 'Foo' }",
            0,
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_has_other_option() {
        let result = create_linter().lint(
            "export default { name: 'Foo', data() { return { count: 0 } } }",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_empty_object() {
        let result = create_linter().lint("export default {}", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_define_component() {
        let result = create_linter().lint(
            "import { defineComponent } from 'vue'\nexport default defineComponent({ name: 'Foo' })",
            0,
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_extra_top_level_statement() {
        let result = create_linter().lint("const x = 1\nexport default { name: 'Foo' }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_spread_in_object() {
        let result = create_linter().lint("export default { ...base, name: 'Foo' }", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_no_default_export() {
        let result = create_linter().lint("const name = 'Foo'", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_offset_applied() {
        let source = "export default { name: 'Foo' }";
        let result = create_linter().lint(source, 70);
        assert_eq!(result.warning_count, 1);
        let object_start = source.find('{').unwrap() as u32 + 70;
        assert_eq!(result.diagnostics[0].start, object_start);
    }
}
