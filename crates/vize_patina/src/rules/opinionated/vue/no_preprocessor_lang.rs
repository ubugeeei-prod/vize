//! vue/no-preprocessor-lang
//!
//! Discourage use of CSS preprocessors (sass, scss, less, stylus) in Vue SFCs.
//!
//! Modern CSS has many features that previously required preprocessors:
//! - CSS Custom Properties (variables)
//! - CSS Nesting (native)
//! - CSS Color Mix
//! - CSS Container Queries
//!
//! Using plain CSS with modern features:
//! - Reduces build complexity
//! - Improves build performance
//! - Enables better tooling support (lightning-css, etc.)
//! - Makes styles more portable

use crate::context::LintContext;
use crate::diagnostic::{Fix, LintDiagnostic, Severity, TextEdit};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-preprocessor-lang",
    description: "Discourage CSS preprocessor usage in favor of modern CSS",
    category: RuleCategory::Recommended,
    fixable: true,
    default_severity: Severity::Warning,
};

/// No preprocessor lang rule
#[derive(Default)]
pub struct NoPreprocessorLang;

impl Rule for NoPreprocessorLang {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        let source = ctx.source;

        // Find all <style tags
        let mut pos = 0;
        while let Some(style_start) = source[pos..].find("<style") {
            let abs_pos = pos + style_start;
            pos = abs_pos + 6;

            // Find the closing >
            if let Some(tag_end) = source[abs_pos..].find('>') {
                let tag_content = &source[abs_pos..abs_pos + tag_end + 1];

                // Check for preprocessor langs
                let preprocessor = if tag_content.contains("lang=\"sass\"")
                    || tag_content.contains("lang='sass'")
                {
                    Some(("sass", "lang=\"sass\""))
                } else if tag_content.contains("lang=\"scss\"")
                    || tag_content.contains("lang='scss'")
                {
                    Some(("scss", "lang=\"scss\""))
                } else if tag_content.contains("lang=\"less\"")
                    || tag_content.contains("lang='less'")
                {
                    Some(("less", "lang=\"less\""))
                } else if tag_content.contains("lang=\"stylus\"")
                    || tag_content.contains("lang='stylus'")
                {
                    Some(("stylus", "lang=\"stylus\""))
                } else if tag_content.contains("lang=\"styl\"")
                    || tag_content.contains("lang='styl'")
                {
                    Some(("styl", "lang=\"styl\""))
                } else {
                    None
                };

                if let Some((_preprocessor_name, lang_attr)) = preprocessor {
                    // Find position of lang attribute for fix
                    let lang_pos = tag_content.find(lang_attr).unwrap_or(0);
                    let lang_start = abs_pos + lang_pos;
                    let lang_end = lang_start + lang_attr.len();

                    // Create fix to remove the lang attribute
                    let fix = Fix::new(
                        "Remove CSS preprocessor",
                        TextEdit::delete(lang_start as u32, (lang_end + 1) as u32), // +1 for trailing space
                    );

                    ctx.report(
                        LintDiagnostic::warn(
                            META.name,
                            "Consider using modern CSS instead of preprocessor",
                            abs_pos as u32,
                            (abs_pos + tag_end + 1) as u32,
                        )
                        .with_help(
                            "Modern CSS supports nesting, variables, and more. Consider migrating to plain CSS.",
                        )
                        .with_fix(fix),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests would need SFC-level testing infrastructure
}
