//! Original configured product output at the native callback host boundary.

use super::{NativeTemplateAttribute, NativeTemplateLintRefusal};
use crate::{HelpLevel, LintDiagnostic, LintResult, Linter, Severity};
use vize_carton::i18n::{Locale, t, t_fmt};
use vize_l0::{Span, String, config::VueVersion};
use vize_l1::markup::NativeLintComponent;

/// Normal configured output from the genuine original bare component.
/// The constructor and unfinished findings are private to the native driver.
/// Inline comment suppression is not inferred; the initial driver refuses it.
pub struct NativeTemplateLintContext<'o, 'a> {
    linter: &'o Linter,
    root: &'o NativeLintComponent<'a>,
    filename: &'o str,
    pub(super) current_rule: &'static str,
    diagnostics: Vec<LintDiagnostic>,
}

impl<'o, 'a> NativeTemplateLintContext<'o, 'a> {
    pub(super) fn new(
        linter: &'o Linter,
        root: &'o NativeLintComponent<'a>,
        filename: &'o str,
    ) -> Self {
        Self {
            linter,
            root,
            filename,
            current_rule: "",
            diagnostics: Vec::new(),
        }
    }

    #[must_use]
    pub fn owner(&self) -> &'o NativeLintComponent<'a> {
        self.root
    }
    #[must_use]
    pub fn filename(&self) -> &'o str {
        self.filename
    }
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.root.component().block().source()
    }
    #[must_use]
    pub fn locale(&self) -> Locale {
        self.linter.locale
    }
    #[must_use]
    pub fn help_level(&self) -> HelpLevel {
        self.linter.help_level
    }
    #[must_use]
    pub fn requested_vue_version(&self) -> Option<VueVersion> {
        self.linter.requested_vue_version
    }
    #[must_use]
    pub fn requested_vapor_mode(&self) -> Option<bool> {
        self.linter.requested_vapor_mode
    }

    /// Report the original standalone root warning at its source point.
    /// The unchanged registered parser's root location is the point 0..0.
    /// The complete original block and configured rule instance stay owned;
    /// labels/fixes are absent for this root-only warning capability.
    pub fn warn_root_with_help(
        &mut self,
        message_key: &str,
        variables: &[(&str, &str)],
        help_key: &str,
    ) {
        let point = self.root.component().block().start();
        self.warn_with_help(Span::new(point, point), message_key, variables, help_key);
    }

    /// Report only from a sealed checked attribute of this actual bare owner.
    /// Equal source buffers/spans cannot substitute for component identity.
    ///
    /// ```compile_fail
    /// use vize_l0::Span;
    /// use vize_patina::native::template::NativeTemplateLintContext;
    /// fn raw(context: &mut NativeTemplateLintContext<'_, '_>) {
    ///     context.warn_attribute_with_help(Span::new(0, 1), "message", &[], "help");
    /// }
    /// ```
    pub fn warn_attribute_with_help(
        &mut self,
        attribute: &NativeTemplateAttribute<'_, 'a>,
        message_key: &str,
        variables: &[(&str, &str)],
        help_key: &str,
    ) -> Result<(), NativeTemplateLintRefusal> {
        if !core::ptr::eq(attribute.original().component(), self.root.component()) {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        self.warn_with_help(attribute.range(), message_key, variables, help_key);
        Ok(())
    }

    fn warn_with_help(
        &mut self,
        range: Span,
        message_key: &str,
        variables: &[(&str, &str)],
        help_key: &str,
    ) {
        let mut diagnostic = LintDiagnostic::warn(
            self.current_rule,
            t_fmt(self.linter.locale, message_key, variables),
            range.start,
            range.end,
        );
        diagnostic.help = self
            .linter
            .help_level
            .process(t(self.linter.locale, help_key).as_ref());
        if let Some(severity) = self.linter.severity_overrides.get(self.current_rule) {
            diagnostic.severity = *severity;
        }
        self.diagnostics.push(diagnostic);
    }

    pub(super) fn finish(self) -> LintResult {
        let error_count = self
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let warning_count = self
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();
        LintResult {
            filename: String::new(self.filename),
            diagnostics: self.diagnostics,
            error_count,
            warning_count,
        }
    }
}

#[cfg(test)]
mod tests;
