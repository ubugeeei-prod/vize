//! vue/no-script-non-standard-lang
//!
//! Discourage non-standard lang values on script blocks.
//!
//! Only standard JavaScript/TypeScript variants are recommended:
//! - ts, tsx (TypeScript)
//! - js, jsx (JavaScript)
//!
//! Non-standard languages like CoffeeScript require additional tooling
//! and are not well supported by modern Vue tooling.
//!
//! ## Allowed Values
//!
//! - `ts` - TypeScript
//! - `tsx` - TypeScript with JSX
//! - `js` - JavaScript (default when no lang specified)
//! - `jsx` - JavaScript with JSX
//!
//! ## Examples
//!
//! Bad:
//! ```vue
//! <script lang="coffee">
//! # CoffeeScript code
//! </script>
//! ```
//!
//! Good:
//! ```vue
//! <script setup lang="ts">
//! // TypeScript code
//! </script>
//!
//! <script lang="tsx">
//! // TypeScript with JSX
//! </script>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-script-non-standard-lang",
    description: "Discourage non-standard script lang values",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Allowed script lang values
const ALLOWED_LANGS: &[&str] = &["ts", "tsx", "js", "jsx", "typescript"];

/// No script non-standard lang rule
#[derive(Default)]
pub struct NoScriptNonStandardLang;

impl Rule for NoScriptNonStandardLang {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        let source = ctx.source;

        // Find all <script tags
        let mut pos = 0;
        while let Some(script_start) = source[pos..].find("<script") {
            let abs_pos = pos + script_start;
            pos = abs_pos + 7;

            // Find the closing >
            if let Some(tag_end) = source[abs_pos..].find('>') {
                let tag_content = &source[abs_pos..abs_pos + tag_end + 1];

                // Extract lang value if present
                if let Some(lang_value) = Self::extract_lang_value(tag_content) {
                    // Check if it's an allowed lang
                    if !ALLOWED_LANGS.contains(&lang_value.to_lowercase().as_str()) {
                        ctx.report(
                            LintDiagnostic::warn(
                                META.name,
                                "Non-standard script lang is discouraged",
                                abs_pos as u32,
                                (abs_pos + tag_end + 1) as u32,
                            )
                            .with_help("Use ts, tsx, js, or jsx for better tooling support"),
                        );
                    }
                }
            }
        }
    }
}

impl NoScriptNonStandardLang {
    /// Extract the lang attribute value from a tag string
    fn extract_lang_value(tag_content: &str) -> Option<&str> {
        // Look for lang="..." or lang='...'
        let patterns = [("lang=\"", '"'), ("lang='", '\'')];

        for (pattern, end_char) in patterns {
            if let Some(start) = tag_content.find(pattern) {
                let value_start = start + pattern.len();
                if let Some(end) = tag_content[value_start..].find(end_char) {
                    return Some(&tag_content[value_start..value_start + end]);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    // Tests would need SFC-level testing infrastructure
}
