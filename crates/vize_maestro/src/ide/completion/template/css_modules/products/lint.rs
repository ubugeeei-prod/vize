//! CSS lint diagnostics, checker options and editor completions for one open revision.
use super::{IdeContext, StyleModuleRefusal, StyleModuleToolProducts, VirtualTsOptions};
use vize_patina::rules::css::{CssLintResult, CssLinter, StyleRuleInput};

impl IdeContext<'_> {
    /// Read the module tool products and the complete diagnostics from the
    /// caller's CSS rule registry in document order. Module classes and
    /// structural rules consume the same original L1 stylesheet.
    pub fn style_module_products_with_lint(
        &self,
        base: &VirtualTsOptions,
        strict: bool,
        resolve_imports: bool,
        linter: &CssLinter,
        respect_disable_comments: bool,
    ) -> Result<Option<(StyleModuleToolProducts, CssLintResult)>, StyleModuleRefusal> {
        let mut lint = CssLintResult::default();
        let modules = self.read_style_module_products::<true>(
            base,
            strict,
            resolve_imports,
            |style, observed| {
                if style.src.is_some() {
                    return;
                }
                let Ok(language) = vize_l1::container::lang::style_lang(style.lang.as_deref())
                else {
                    return;
                };
                if !linter.reads(language) || style.content.trim().is_empty() {
                    return;
                }
                let input = match observed {
                    Some(style) => StyleRuleInput::Observed(style),
                    None => StyleRuleInput::Inline {
                        source: style.content.as_ref(),
                        offset: style.loc.start,
                        language,
                    },
                };
                let result = linter.lint_file_style(input, respect_disable_comments);
                if lint.diagnostics.is_empty() {
                    lint.diagnostics = result.diagnostics;
                    // The class-name pass will use its own scratch space. Keep
                    // the first module's emitted rows across that pass at their
                    // actual length, without retaining the rule buffer's slack.
                    if observed.is_some() {
                        lint.diagnostics.shrink_to_fit();
                    }
                } else {
                    lint.diagnostics.extend(result.diagnostics);
                }
                lint.error_count += result.error_count;
                lint.warning_count += result.warning_count;
            },
        )?;
        Ok(modules.map(|modules| (modules, lint)))
    }
}
