//! Pending configured diagnostics can name only this owner's checked attributes.

use super::NativeSfcTemplateOwner;
use crate::native::template::{
    NativeTemplateAttribute, NativeTemplateAttributeKind, NativeTemplateLintRefusal as Refusal,
};
use crate::{HelpLevel, LintDiagnostic, LintResult, Linter, Severity};
use vize_carton::i18n::{Locale, t, t_fmt};
use vize_l0::{String, config::VueVersion};

#[cfg(test)]
mod tests;

pub struct NativeSfcTemplateLintContext<'o, 'a> {
    linter: &'o Linter,
    owner: &'o NativeSfcTemplateOwner<'a>,
    filename: &'o str,
    pub(super) current_rule: &'static str,
    pub(super) default_severity: Severity,
    diagnostics: Vec<LintDiagnostic>,
}

impl<'o, 'a> NativeSfcTemplateLintContext<'o, 'a> {
    pub(super) fn new(
        linter: &'o Linter,
        owner: &'o NativeSfcTemplateOwner<'a>,
        filename: &'o str,
    ) -> Self {
        Self {
            linter,
            owner,
            filename,
            current_rule: "",
            default_severity: Severity::Error,
            diagnostics: Vec::new(),
        }
    }

    #[must_use]
    pub fn owner(&self) -> &'o NativeSfcTemplateOwner<'a> {
        self.owner
    }
    #[must_use]
    pub fn filename(&self) -> &'o str {
        self.filename
    }
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.owner.source()
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

    /// Report the name of a checked static attribute owned by this original
    /// selected component. No caller range or detached name can mint output.
    ///
    /// ```compile_fail
    /// use vize_l0::Span;
    /// use vize_patina::native::sfc::NativeSfcTemplateLintContext;
    /// fn raw(context: &mut NativeSfcTemplateLintContext<'_, '_>) {
    ///     context.error_attribute_name_with_help(Span::new(15, 19), "message", &[], "help");
    /// }
    /// ```
    pub fn error_attribute_name_with_help(
        &mut self,
        attribute: &NativeTemplateAttribute<'_, 'a>,
        message_key: &str,
        variables: &[(&str, &str)],
        help_key: &str,
    ) -> Result<(), Refusal> {
        let component = self.owner.template().component();
        if !core::ptr::eq(attribute.original().component(), component) {
            return Err(Refusal::SourceMismatch);
        }
        let NativeTemplateAttributeKind::Static { name, .. } = attribute.kind() else {
            return Err(Refusal::UnsupportedAttribute {
                span: attribute.range(),
            });
        };
        if attribute.original().surface().name.text != name {
            return Err(Refusal::SourceMismatch);
        }
        let range = crate::native::attribute::span(component.block(), name)?;
        if range.start != attribute.range().start || range.end > attribute.range().end {
            return Err(Refusal::SourceMismatch);
        }
        let mut diagnostic = LintDiagnostic::error(
            self.current_rule,
            t_fmt(self.linter.locale, message_key, variables),
            range.start,
            range.end,
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
