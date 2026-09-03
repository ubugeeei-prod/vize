//! script/no-internal-imports
//!
//! Disallow importing from Vue internal modules.
//!
//! ## Rationale
//!
//! Vue.js has internal modules that are not part of the public API.
//! Importing from these modules is dangerous as they may change without
//! notice between minor/patch versions.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! import { foo } from '@vue/runtime-core/dist/runtime-core.esm-bundler'
//! import { bar } from 'vue/dist/vue.esm-bundler'
//! ```
//!
//! ### Valid
//! ```ts
//! import { ref, computed } from 'vue'
//! ```

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{ImportDeclaration, Program, Statement};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-internal-imports",
    description: "Disallow importing from Vue internal modules",
    default_severity: Severity::Error,
};

/// Internal import patterns that should be forbidden.
const INTERNAL_PATTERNS: &[&str] = &[
    "/dist/",      // Any dist import
    "/src/",       // Source imports
    "/esm/",       // ESM subpath
    "vue.esm",     // Direct bundle imports
    "vue.cjs",     // CJS bundle imports
    "vue.runtime", // Runtime bundle imports
];

/// Disallow importing from Vue internal modules
pub struct NoInternalImports;

impl ScriptRule for NoInternalImports {
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
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            check_import(import, source, offset, result);
        }
    }
}

fn check_import(
    import: &ImportDeclaration<'_>,
    source: &str,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    let specifier = import.source.value.as_str();
    if !is_vue_import(specifier) || !is_internal_import(specifier) {
        return;
    }

    let (span_start, span_end) = specifier_span(import, source);
    let start = offset as u32 + span_start;
    let end = offset as u32 + span_end;
    result.add_diagnostic(
        LintDiagnostic::error(
            META.name,
            "Importing from internal Vue module is forbidden",
            start,
            end,
        )
        .with_help("Import from 'vue' directly instead of internal modules"),
    );
}

fn is_vue_import(specifier: &str) -> bool {
    specifier == "vue"
        || specifier.starts_with("vue/")
        || specifier.starts_with("vue.")
        || specifier.starts_with("@vue/")
}

fn is_internal_import(specifier: &str) -> bool {
    INTERNAL_PATTERNS
        .iter()
        .any(|pattern| specifier.contains(pattern))
}

fn specifier_span(import: &ImportDeclaration<'_>, source: &str) -> (u32, u32) {
    let start = import.source.span.start as usize;
    let end = import.source.span.end as usize;
    let bytes = source.as_bytes();
    if let (Some(quote_start), Some(quote_end)) = (bytes.get(start), bytes.get(end - 1))
        && matches!(quote_start, b'\'' | b'"')
        && quote_start == quote_end
    {
        return ((start + 1) as u32, (end - 1) as u32);
    }
    (import.source.span.start, import.source.span.end)
}

#[cfg(test)]
mod tests {
    use super::{NoInternalImports, ScriptLintResult, ScriptRule};

    #[test]
    fn test_valid_vue_import() {
        let source = "import { ref } from 'vue'";
        let rule = NoInternalImports;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_dist_import() {
        let source = "import { ref } from 'vue/dist/vue.esm-bundler.js'";
        let rule = NoInternalImports;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_invalid_runtime_core_dist() {
        let source = "import { ref } from '@vue/runtime-core/dist/runtime-core.esm-bundler.js'";
        let rule = NoInternalImports;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_valid_vue_package_import() {
        // Importing from @vue/* packages (even if not recommended) is allowed
        // The prefer-import-from-vue rule handles that case
        let source = "import { ref } from '@vue/reactivity'";
        let rule = NoInternalImports;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_relative_vue_sfc_import() {
        let source = "import Button from '../src/shared/AppButton.vue'";
        let rule = NoInternalImports;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_non_vue_import() {
        let source = "import { foo } from 'lodash/dist/lodash.js'";
        let rule = NoInternalImports;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_double_quote_import() {
        let source = r#"import { ref } from "vue/dist/vue.esm-bundler.js""#;
        let rule = NoInternalImports;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_vue_esm_pattern() {
        let source = "import { ref } from 'vue.esm.js'";
        let rule = NoInternalImports;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_side_effect_internal_import() {
        let source = "import 'vue/dist/vue.esm-bundler.js'";
        let rule = NoInternalImports;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 1);
    }

    #[test]
    fn test_string_literal_false_positive() {
        let source = "const specifier = \"from 'vue/dist/vue.esm-bundler.js'\"";
        let rule = NoInternalImports;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.error_count, 0);
    }
}
