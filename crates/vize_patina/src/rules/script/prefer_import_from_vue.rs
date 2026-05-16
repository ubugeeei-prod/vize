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

use memchr::memmem;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};

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
    fn check(&self, source: &str, offset: usize, result: &mut ScriptLintResult) {
        let bytes = source.as_bytes();

        // Early bailout: no @vue imports
        if memmem::find(bytes, b"@vue/").is_none() {
            return;
        }

        // Find all "from" keywords and check the module specifier
        let from_finder = memmem::Finder::new(b"from");
        let mut search_start = 0;

        while let Some(from_pos) = from_finder.find(&bytes[search_start..]) {
            let abs_from_pos = search_start + from_pos;

            // Skip whitespace after "from"
            let after_from = &bytes[abs_from_pos + 4..];
            let trimmed_start = skip_whitespace(after_from);

            if trimmed_start >= after_from.len() {
                search_start = abs_from_pos + 4;
                continue;
            }

            let quote = after_from[trimmed_start];
            if quote != b'\'' && quote != b'"' {
                search_start = abs_from_pos + 4;
                continue;
            }

            // Find the closing quote
            let specifier_start = trimmed_start + 1;
            let Some(quote_end) = memchr::memchr(quote, &after_from[specifier_start..]) else {
                search_start = abs_from_pos + 4;
                continue;
            };

            let module_specifier = &after_from[specifier_start..specifier_start + quote_end];

            // Check if it matches any internal package
            // SAFETY: module_specifier comes from source which is valid UTF-8
            let specifier_str = unsafe { std::str::from_utf8_unchecked(module_specifier) };

            for pkg in INTERNAL_PACKAGES {
                if specifier_str == *pkg {
                    // Calculate the "from" to closing quote range
                    let pattern_start = offset + abs_from_pos;
                    let pattern_end = offset + abs_from_pos + 4 + specifier_start + quote_end + 1; // +1 for closing quote

                    // Create fix string
                    let fix_str = if trimmed_start > 0 {
                        if quote == b'\'' {
                            "from 'vue'"
                        } else {
                            "from \"vue\""
                        }
                    } else if quote == b'\'' {
                        "from'vue'"
                    } else {
                        "from\"vue\""
                    };

                    result.add_diagnostic(
                        LintDiagnostic::warn(
                            META.name,
                            format!("Import from '{}' should be replaced with 'vue'", pkg),
                            pattern_start as u32,
                            pattern_end as u32,
                        )
                        .with_help("Import from 'vue' directly for better compatibility")
                        .with_fix(Fix::new(
                            "Replace with 'vue'",
                            TextEdit::new(pattern_start as u32, pattern_end as u32, fix_str),
                        )),
                    );
                    break;
                }
            }

            search_start = abs_from_pos + 4 + specifier_start + quote_end;
        }
    }
}

/// Skip ASCII whitespace and return the offset
#[inline]
fn skip_whitespace(bytes: &[u8]) -> usize {
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
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
}
