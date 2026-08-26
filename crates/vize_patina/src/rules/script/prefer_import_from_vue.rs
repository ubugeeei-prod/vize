//! script/prefer-import-from-vue
//!
//! Prefer importing from 'vue' instead of '@vue/*' internal packages.
//!
//! ## Rationale
//!
//! While Vue.js is split into multiple packages internally, end users should
//! always import from 'vue' directly. The internal packages like '@vue/runtime-core'
//! and '@vue/runtime-dom' are implementation details and may change between versions.
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
//! ```

#![allow(clippy::disallowed_macros)]

use oxc_ast::ast::{ImportDeclaration, Program, Statement};
use oxc_span::GetSpan;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use vize_s0::String;

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

    let Some(from_start) = import_from_start(import, source) else {
        return;
    };
    let source_start = import.source.span.start as usize;
    let pattern_start = offset + from_start;
    let pattern_end = offset + import.source.span.end as usize;
    let fix_str = replacement_from_clause(source, from_start, source_start);

    result.add_diagnostic(
        LintDiagnostic::warn(
            META.name,
            format!("Import from '{}' should be replaced with 'vue'", specifier),
            pattern_start as u32,
            pattern_end as u32,
        )
        .with_help("Import from 'vue' directly for better compatibility")
        .with_fix(Fix::new(
            "Replace with 'vue'",
            TextEdit::new(pattern_start as u32, pattern_end as u32, fix_str),
        )),
    );
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
