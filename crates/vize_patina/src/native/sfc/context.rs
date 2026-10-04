//! Configured pending findings sealed to this genuine original SFC owner.

use super::{NativeSfcLintOwner, NativeSfcLintRefusal, NativeSfcSetup};
use crate::{HelpLevel, LintDiagnostic, LintResult, Linter, Severity};
use vize_carton::i18n::{Locale, t, t_fmt};
use vize_l0::{String, config::VueVersion};

pub struct NativeSfcLintContext<'o, 'a> {
    linter: &'o Linter,
    owner: &'o NativeSfcLintOwner<'a>,
    filename: &'o str,
    pub(super) current_rule: &'static str,
    pub(super) default_severity: Severity,
    diagnostics: Vec<LintDiagnostic>,
}

impl<'o, 'a> NativeSfcLintContext<'o, 'a> {
    pub(super) fn new(
        linter: &'o Linter,
        owner: &'o NativeSfcLintOwner<'a>,
        filename: &'o str,
    ) -> Self {
        Self {
            linter,
            owner,
            filename,
            current_rule: "",
            default_severity: Severity::Warning,
            diagnostics: Vec::new(),
        }
    }
    #[must_use]
    pub fn owner(&self) -> &'o NativeSfcLintOwner<'a> {
        self.owner
    }
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.owner.source()
    }
    #[must_use]
    pub fn filename(&self) -> &'o str {
        self.filename
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

    /// Report at the original setup block's source point with configured
    /// locale/help/severity. Only this owner's sealed whole setup grants it.
    /// Arbitrary spans and a foreign same-buffer setup cannot mint findings.
    ///
    /// ```compile_fail
    /// use vize_l0::Span;
    /// use vize_patina::native::sfc::NativeSfcLintContext;
    /// fn raw(context: &mut NativeSfcLintContext<'_, '_>) {
    ///     context.warn_setup_with_help(Span::new(0, 1), "message", &[], "help");
    /// }
    /// ```
    /// A raw same-buffer semantic view is not the sealed SFC owner capability:
    /// ```compile_fail
    /// use vize_l2::lang::js::VueSetup;
    /// use vize_patina::native::sfc::NativeSfcLintContext;
    /// fn detached<'a>(context: &mut NativeSfcLintContext<'_, 'a>,
    ///     setup: &VueSetup<'_, '_, '_, 'a>) {
    ///     context.warn_setup_with_help(setup, "message", &[], "help");
    /// }
    /// ```
    pub fn warn_setup_with_help(
        &mut self,
        setup: &NativeSfcSetup<'_, 'a>,
        message_key: &str,
        variables: &[(&str, &str)],
        help_key: &str,
    ) -> Result<(), NativeSfcLintRefusal> {
        if !core::ptr::eq(setup.owner(), self.owner) {
            return Err(NativeSfcLintRefusal::SourceMismatch);
        }
        let point = setup.semantic().source().start();
        let mut diagnostic = LintDiagnostic::warn(
            self.current_rule,
            t_fmt(self.linter.locale, message_key, variables),
            point,
            point,
        );
        diagnostic.severity = self
            .linter
            .severity_overrides
            .get(self.current_rule)
            .copied()
            .unwrap_or(self.default_severity);
        diagnostic.help = self
            .linter
            .help_level
            .process(t(self.linter.locale, help_key).as_ref());
        self.diagnostics.push(diagnostic);
        Ok(())
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
