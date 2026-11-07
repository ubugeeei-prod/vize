//! script/prefer-define-options
//!
//! Prefer the `defineOptions()` macro over a default export in a plain `<script>`
//! block paired with `<script setup>`.
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
//! has a default export, suggesting the move to `defineOptions`.
//!
//! ## Examples
//!
//! ### Invalid (plain `<script>` carrying only name/inheritAttrs beside setup)
//! ```vue
//! <script lang="ts">
//! export default { name: 'MyComponent', inheritAttrs: false }
//! </script>
//! <script setup lang="ts">
//! const ready = true
//! </script>
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

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{Program, Statement};
use oxc_span::Span;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/prefer-define-options",
    description: "Prefer defineOptions() over a default export paired with <script setup>",
    default_severity: Severity::Warning,
};

const MESSAGE: &str =
    "Prefer defineOptions() over a separate <script> block paired with <script setup>.";
const HELP: &str = "Move the default export into a `defineOptions(...)` call in <script setup>. \
     Remove the plain <script> block only when it has no other content.";

/// Prefer `defineOptions()` over a plain-script default export next to setup.
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
        _program: &'a Program<'a>,
        _source: &str,
        _offset: usize,
        _result: &mut ScriptLintResult,
    ) {
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        // eslint-plugin-vue only offers this migration when a sibling
        // <script setup> exists. A normal Options API component is not a
        // defineOptions candidate.
        if !sfc.is_sfc || sfc.is_script_setup || sfc.sibling_program.is_none() {
            return;
        }
        let Some(span) = default_export_span(program) else {
            return;
        };
        let start = offset as u32 + span.start;
        let end = offset as u32 + span.end;
        result.add_diagnostic(
            LintDiagnostic::warn(META.name, MESSAGE, start, end)
                .with_label("default export can move to defineOptions", start, end)
                .with_help(HELP),
        );
    }
}

fn default_export_span(program: &Program<'_>) -> Option<Span> {
    for statement in &program.body {
        if let Statement::ExportDefaultDeclaration(export) = statement {
            return Some(export.span);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::linter::Linter;

    fn lint_with_setup(source: &str) -> crate::linter::LintResult {
        let sfc = format!("<script>{source}</script><script setup>const ready = true</script>");
        Linter::new()
            .with_enabled_rules(Some(vec!["script/prefer-define-options".into()]))
            .lint_sfc(&sfc, "Probe.vue")
    }

    #[test]
    fn test_invalid_name_and_inherit_attrs() {
        let result = lint_with_setup("export default { name: 'MyComponent', inheritAttrs: false }");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_name_only() {
        let result = lint_with_setup("export default { name: 'Foo' }");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_inherit_attrs_only() {
        let result = lint_with_setup("export default { inheritAttrs: false }");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_with_imports_present() {
        let result = lint_with_setup("import { foo } from './foo'\nexport default { name: 'Foo' }");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_has_other_option() {
        let result =
            lint_with_setup("export default { name: 'Foo', data() { return { count: 0 } } }");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_empty_object() {
        let result = lint_with_setup("export default {}");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_define_component() {
        let result = lint_with_setup(
            "import { defineComponent } from 'vue'\nexport default defineComponent({ name: 'Foo' })",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_extra_top_level_statement() {
        let result = lint_with_setup("const x = 1\nexport default { name: 'Foo' }");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_spread_in_object() {
        let result = lint_with_setup("export default { ...base, name: 'Foo' }");
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_no_default_export() {
        let result = lint_with_setup("const name = 'Foo'");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_plain_script_without_script_setup_is_valid() {
        let result = Linter::new()
            .with_enabled_rules(Some(vec!["script/prefer-define-options".into()]))
            .lint_sfc(
                "<script>export default { name: 'NormalOptionsComponent' }</script>",
                "Probe.vue",
            );
        assert_eq!(result.warning_count, 0);
    }
}
