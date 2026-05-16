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

use memchr::memmem;

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-internal-imports",
    description: "Disallow importing from Vue internal modules",
    default_severity: Severity::Error,
};

/// Internal import patterns that should be forbidden (as byte slices for fast comparison)
const INTERNAL_PATTERNS: &[&[u8]] = &[
    b"/dist/",      // Any dist import
    b"/src/",       // Source imports
    b"/esm/",       // ESM subpath
    b"vue.esm",     // Direct bundle imports
    b"vue.cjs",     // CJS bundle imports
    b"vue.runtime", // Runtime bundle imports
];

/// Disallow importing from Vue internal modules
pub struct NoInternalImports;

impl ScriptRule for NoInternalImports {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn check(&self, source: &str, offset: usize, result: &mut ScriptLintResult) {
        let bytes = source.as_bytes();

        // Early bailout: no vue-related imports possible
        if memmem::find(bytes, b"vue").is_none() {
            return;
        }

        // Find all "from" statements efficiently
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

            // Only check Vue-related imports
            if !contains_vue(module_specifier) {
                search_start = abs_from_pos + 4 + specifier_start + quote_end;
                continue;
            }

            // Check for internal patterns
            for pattern in INTERNAL_PATTERNS {
                if memmem::find(module_specifier, pattern).is_some() {
                    // Calculate absolute position in source
                    let spec_abs_start = abs_from_pos + 4 + specifier_start;
                    let start = offset + spec_abs_start;
                    let end = start + module_specifier.len();

                    result.add_diagnostic(
                        LintDiagnostic::error(
                            META.name,
                            "Importing from internal Vue module is forbidden",
                            start as u32,
                            end as u32,
                        )
                        .with_help("Import from 'vue' directly instead of internal modules"),
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

/// Check if bytes contain "vue" (case-sensitive)
#[inline]
fn contains_vue(bytes: &[u8]) -> bool {
    memmem::find(bytes, b"vue").is_some() || memmem::find(bytes, b"@vue/").is_some()
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
}
