//! script/prefer-import-from-vue
//!
//! Prefer importing from 'vue' instead of '@vue/*' internal packages.
//!
//! ## Rationale
//!
//! Vue.js is split into multiple packages internally. Names publicly re-exported
//! by 'vue' should use that entry point. Private helpers that are not available
//! from 'vue' must retain their actual source. Namespace, default, and mixed
//! public/private imports keep the advisory without an unsafe automatic rewrite.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! import { ref } from '@vue/runtime-core'
//! import { h } from '@vue/runtime-dom'
//! ```
//!
//! ### Valid
//! ```ts
//! import { ref, h } from 'vue'
//! import { isString } from '@vue/shared'
//! ```

use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, Program, Statement};
use oxc_span::GetSpan;
use vize_l0::cstr;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use vize_l0::String;

mod public_exports;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/prefer-import-from-vue",
    description: "Prefer importing from 'vue' instead of internal packages",
    default_severity: Severity::Warning,
};

/// Internal Vue packages that should be replaced with 'vue'
const INTERNAL_PACKAGES: &[&str] = &[
    "@vue/runtime-core",
    "@vue/runtime-dom",
    "@vue/reactivity",
    "@vue/shared",
];

/// Prefer importing from 'vue' instead of internal packages
pub struct PreferImportFromVue;

impl ScriptRule for PreferImportFromVue {
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
    if !INTERNAL_PACKAGES.contains(&specifier) {
        return;
    }

    let mut only_named = true;
    let mut has_specifier = false;
    let mut has_public_name = false;
    let mut safe_to_fix = true;
    for binding in import.specifiers.iter().flatten() {
        has_specifier = true;
        match binding {
            ImportDeclarationSpecifier::ImportSpecifier(binding) => {
                let public = public_exports::contains(binding.imported.name().as_str());
                has_public_name |= public;
                safe_to_fix &= public;
            }
            _ => {
                only_named = false;
                safe_to_fix = false;
            }
        }
    }
    if has_specifier && only_named && !has_public_name {
        return;
    }

    let Some(from_start) = import_from_start(import, source) else {
        return;
    };
    let source_start = import.source.span.start as usize;
    let pattern_start = offset + from_start;
    let pattern_end = offset + import.source.span.end as usize;
    let diagnostic = LintDiagnostic::warn(
        META.name,
        cstr!("Import from '{}' should be replaced with 'vue'", specifier),
        pattern_start as u32,
        pattern_end as u32,
    )
    .with_help("Import from 'vue' directly for better compatibility");
    let diagnostic = if safe_to_fix {
        diagnostic.with_fix(Fix::new(
            "Replace with 'vue'",
            TextEdit::new(
                pattern_start as u32,
                pattern_end as u32,
                replacement_from_clause(source, from_start, source_start),
            ),
        ))
    } else {
        diagnostic
    };
    result.add_diagnostic(diagnostic);
}

fn import_from_start(import: &ImportDeclaration<'_>, source: &str) -> Option<usize> {
    let import_start = import.span().start as usize;
    let source_start = import.source.span.start as usize;
    source
        .get(import_start..source_start)?
        .rfind("from")
        .map(|pos| import_start + pos)
}

fn replacement_from_clause(source: &str, from_start: usize, source_start: usize) -> String {
    let separator = source.get(from_start + 4..source_start).unwrap_or(" ");
    let quote = source
        .as_bytes()
        .get(source_start)
        .copied()
        .filter(|quote| matches!(quote, b'\'' | b'"'))
        .unwrap_or(b'\'') as char;
    let mut replacement = String::from("from");
    replacement.push_str(separator);
    replacement.push(quote);
    replacement.push_str("vue");
    replacement.push(quote);
    replacement
}

#[cfg(test)]
mod tests {
    use super::{PreferImportFromVue, ScriptLintResult, ScriptRule};

    #[test]
    fn test_valid_vue_import() {
        let source = "import { ref } from 'vue'";
        let rule = PreferImportFromVue;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_runtime_core_import() {
        let source = "import { ref } from '@vue/runtime-core'";
        let rule = PreferImportFromVue;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_runtime_dom_import() {
        let source = "import { h } from '@vue/runtime-dom'";
        let rule = PreferImportFromVue;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_invalid_reactivity_import() {
        let source = "import { reactive } from '@vue/reactivity'";
        let rule = PreferImportFromVue;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_multiple_invalid_imports() {
        let source = r#"
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
"#;
        let rule = PreferImportFromVue;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 2);
    }

    #[test]
    fn test_has_fix() {
        let source = "import { ref } from '@vue/runtime-core'";
        let rule = PreferImportFromVue;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert!(result.diagnostics[0].fix.is_some());
    }

    #[test]
    fn test_double_quote_import() {
        let source = r#"import { ref } from "@vue/runtime-core""#;
        let rule = PreferImportFromVue;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_no_space_import() {
        let source = "import { ref } from'@vue/runtime-core'";
        let rule = PreferImportFromVue;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_ignores_from_text_inside_string() {
        let source = "const code = \"import { ref } from '@vue/runtime-core'\"";
        let rule = PreferImportFromVue;
        let mut result = ScriptLintResult::default();
        rule.check(source, 0, &mut result);
        assert_eq!(result.warning_count, 0);
    }
}
