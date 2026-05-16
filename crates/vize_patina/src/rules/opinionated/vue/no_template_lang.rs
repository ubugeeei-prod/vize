//! vue/no-template-lang
//!
//! Discourage use of lang attribute on template block.
//!
//! Using template preprocessors like Pug or Jade:
//! - Reduces portability of components
//! - Requires additional build tooling
//! - Makes it harder to use Vue devtools
//! - Can cause issues with IDE support
//!
//! Modern Vue templates with native HTML are more maintainable
//! and have better tooling support.
//!
//! ## Examples
//!
//! Bad:
//! ```vue
//! <template lang="pug">
//! div.container
//!   h1 Hello
//! </template>
//! ```
//!
//! Good:
//! ```vue
//! <template>
//!   <div class="container">
//!     <h1>Hello</h1>
//!   </div>
//! </template>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-template-lang",
    description: "Discourage lang attribute on template block",
    category: RuleCategory::Recommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// No template lang rule
#[derive(Default)]
pub struct NoTemplateLang;

impl Rule for NoTemplateLang {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        let source = ctx.source;

        // Find <template tag
        if let Some(template_start) = source.find("<template") {
            // Find the closing >
            if let Some(tag_end) = source[template_start..].find('>') {
                let tag_content = &source[template_start..template_start + tag_end + 1];

                // Check for lang attribute
                let lang_patterns = [
                    ("lang=\"pug\"", "pug"),
                    ("lang='pug'", "pug"),
                    ("lang=\"jade\"", "jade"),
                    ("lang='jade'", "jade"),
                    ("lang=\"slm\"", "slm"),
                    ("lang='slm'", "slm"),
                    ("lang=\"haml\"", "haml"),
                    ("lang='haml'", "haml"),
                ];

                for (pattern, _lang_name) in lang_patterns {
                    if tag_content.contains(pattern) {
                        let lang_pos = tag_content.find(pattern).unwrap_or(0);
                        let lang_start = template_start + lang_pos;
                        let lang_end = lang_start + pattern.len();

                        // Create fix to remove the lang attribute
                        let fix = Fix::new(
                            "Remove template preprocessor",
                            TextEdit::delete(lang_start as u32, (lang_end + 1) as u32), // +1 for trailing space
                        );

                        ctx.report(
                            LintDiagnostic::warn(
                                META.name,
                                "Avoid using template preprocessor for better tooling support",
                                template_start as u32,
                                (template_start + tag_end + 1) as u32,
                            )
                            .with_help("Use native HTML templates for better IDE support and maintainability")
                            .with_fix(fix),
                        );
                        break;
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
