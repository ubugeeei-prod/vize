//! CSS lint diagnostics, checker options and editor completions for one open revision.
use super::{IdeContext, StyleModuleRefusal, StyleModuleToolProducts, VirtualTsOptions};
use vize_patina::rules::css::{CssLintResult, CssLinter};

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
                let result = match observed {
                    Some(observed) => linter.lint_observed(observed, respect_disable_comments),
                    None => linter.lint_block(
                        style.content.as_ref(),
                        style.loc.start,
                        language,
                        respect_disable_comments,
                    ),
                };
                lint.diagnostics.extend(result.diagnostics);
                lint.error_count += result.error_count;
                lint.warning_count += result.warning_count;
            },
        )?;
        Ok(modules.map(|modules| (modules, lint)))
    }
}
