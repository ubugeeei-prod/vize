//! vue/no-src-attribute
//!
//! Discourage use of src attribute on SFC blocks.
//!
//! Using src attribute to reference external files:
//! - Splits component logic across multiple files
//! - Makes components harder to understand at a glance
//! - Reduces the benefit of Single File Components
//! - Can cause issues with build tools and IDE support
//!
//! Keep all component code in the same .vue file for better
//! maintainability and developer experience.
//!
//! ## Examples
//!
//! Bad:
//! ```vue
//! <template src="./template.html"></template>
//! <script src="./script.ts"></script>
//! <style src="./style.css"></style>
//! ```
//!
//! Good:
//! ```vue
//! <template>
//!   <div>Hello</div>
//! </template>
//!
//! <script setup lang="ts">
//! // Component logic here
//! </script>
//!
//! <style scoped>
//! .container { color: red; }
//! </style>
//! ```

#![allow(clippy::disallowed_macros)]

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-src-attribute",
    description: "Discourage src attribute on SFC blocks",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// No src attribute rule
#[derive(Default)]
pub struct NoSrcAttribute;

impl Rule for NoSrcAttribute {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        let source = ctx.source;

        // Check each SFC block type
        let block_types = ["template", "script", "style"];

        for block_type in block_types {
            let pattern = format!("<{}", block_type);
            let mut pos = 0;

            while let Some(block_start) = source[pos..].find(&pattern) {
                let abs_pos = pos + block_start;
                pos = abs_pos + pattern.len();

                // Find the closing >
                if let Some(tag_end) = source[abs_pos..].find('>') {
                    let tag_content = &source[abs_pos..abs_pos + tag_end + 1];

                    // Check for src attribute
                    let src_patterns = [
                        "src=\"", "src='", ":src=\"", // dynamic binding
                        ":src='",
                    ];

                    for src_pattern in src_patterns {
                        if tag_content.contains(src_pattern) {
                            ctx.report(
                                LintDiagnostic::warn(
                                    META.name,
                                    "Avoid using src attribute on SFC blocks",
                                    abs_pos as u32,
                                    (abs_pos + tag_end + 1) as u32,
                                )
                                .with_help("Keep all component code in the same .vue file for better maintainability"),
                            );
                            break;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests would need SFC-level testing infrastructure
}
