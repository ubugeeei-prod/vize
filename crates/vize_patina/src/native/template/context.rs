//! Original configured product output at the native callback host boundary.

use super::{
    NativeTemplateAttribute, NativeTemplateClosedSelect, NativeTemplateClosedTextarea,
    NativeTemplateElement, NativeTemplateFirstOption, NativeTemplateLintRefusal,
    NativeTemplateOptionValue, NativeTemplateOwnerProfile,
};
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
    pub(super) current_owner_profile: NativeTemplateOwnerProfile,
    pub(super) current_callback: Option<usize>,
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
            current_owner_profile: NativeTemplateOwnerProfile::Strict,
            current_callback: None,
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

    /// Report a checked whole attribute with an Error default. This grants no
    /// raw range or foreign-owner authority; configured severity still wins.
    ///
    /// ```compile_fail
    /// use vize_l0::Span;
    /// use vize_patina::native::template::NativeTemplateLintContext;
    /// fn raw(context: &mut NativeTemplateLintContext<'_, '_>) {
    ///     context.error_attribute_with_help(Span::new(0, 1), "message", &[], "help");
    /// }
    /// ```
    pub fn error_attribute_with_help(
        &mut self,
        attribute: &NativeTemplateAttribute<'_, 'a>,
        message_key: &str,
        variables: &[(&str, &str)],
        help_key: &str,
    ) -> Result<(), NativeTemplateLintRefusal> {
        if !core::ptr::eq(attribute.original().component(), self.root.component()) {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        self.report_with_help(
            Severity::Error,
            attribute.range(),
            message_key,
            variables,
            help_key,
        );
        Ok(())
    }

    /// Report the complete checked opening of a genuine void/self-closing
    /// element. A caller's span, a foreign owner or a nonvoid extent cannot
    /// stand in for this original checked header.
    ///
    /// ```compile_fail
    /// use vize_l0::Span;
    /// use vize_patina::native::template::NativeTemplateLintContext;
    /// fn raw(context: &mut NativeTemplateLintContext<'_, '_>) {
    ///     context.warn_void_element_with_help(Span::new(0, 7), "message", &[], "help");
    /// }
    /// ```
    pub fn warn_void_element_with_help(
        &mut self,
        element: &NativeTemplateElement<'_, 'a>,
        message_key: &str,
        variables: &[(&str, &str)],
        help_key: &str,
    ) -> Result<(), NativeTemplateLintRefusal> {
        let original = element.original();
        let range = self.element_opening_range(element)?;
        if !matches!(
            original.surface().close,
            vize_l1::markup::ElementClose::NotExpected
        ) {
            return Err(NativeTemplateLintRefusal::UnsupportedContext { span: range });
        }
        self.warn_with_help(range, message_key, variables, help_key);
        Ok(())
    }

    /// Read the complete opening of this context's genuine checked element.
    /// This observation grants no diagnostic or source-grammar admission.
    pub fn element_opening_range(
        &self,
        element: &NativeTemplateElement<'_, 'a>,
    ) -> Result<Span, NativeTemplateLintRefusal> {
        let original = element.original();
        if !core::ptr::eq(original.component(), self.root.component()) {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        Ok(crate::native::header::opening_range(original)?)
    }

    /// Report only this callback's admitted closed select, at its opening.
    /// The grammar proof cannot be replaced by a raw range or foreign owner.
    pub fn warn_select_with_help(
        &mut self,
        owner: &NativeTemplateClosedSelect<'_, '_, 'a>,
        message_key: &str,
        variables: &[(&str, &str)],
        help_key: &str,
    ) -> Result<(), NativeTemplateLintRefusal> {
        let range = self.select_opening_range(owner)?;
        self.warn_with_help(range, message_key, variables, help_key);
        Ok(())
    }

    /// Check both original ownership and this actual callback's capability.
    /// Even a hidden/labelled select must establish this before returning clean.
    pub fn select_opening_range(
        &self,
        owner: &NativeTemplateClosedSelect<'_, '_, 'a>,
    ) -> Result<Span, NativeTemplateLintRefusal> {
        let range = self.element_opening_range(owner.element)?;
        if !self.current_owner_profile.allows_select() {
            return Err(NativeTemplateLintRefusal::UnsupportedContext { span: range });
        }
        Ok(range)
    }

    /// Authenticate even clean textarea outcomes against the frozen callback.
    pub fn textarea_opening_range(
        &self,
        owner: &NativeTemplateClosedTextarea<'_, '_, 'a>,
    ) -> Result<Span, NativeTemplateLintRefusal> {
        let range = self.element_opening_range(owner.element)?;
        if self.current_owner_profile != NativeTemplateOwnerProfile::ClosedSelectAndTextarea {
            return Err(NativeTemplateLintRefusal::UnsupportedContext { span: range });
        }
        Ok(range)
    }

    /// Report the original textarea opening, never its opaque body extent.
    pub fn warn_textarea_with_help(
        &mut self,
        owner: &NativeTemplateClosedTextarea<'_, '_, 'a>,
        message_key: &str,
        variables: &[(&str, &str)],
        help_key: &str,
    ) -> Result<(), NativeTemplateLintRefusal> {
        let range = self.textarea_opening_range(owner)?;
        self.warn_with_help(range, message_key, variables, help_key);
        Ok(())
    }

    /// Issue the completed select's first-header token to this callback.
    /// No first option means proven absence in the complete bounded grammar.
    pub fn first_option<'s, 'e>(
        &self,
        owner: &NativeTemplateClosedSelect<'s, 'e, 'a>,
    ) -> Result<Option<NativeTemplateFirstOption<'s, 'e, 'a>>, NativeTemplateLintRefusal> {
        let range = self.select_opening_range(owner)?;
        let issuer = self.callback_issuer(range)?;
        Ok(owner
            .element
            .first_option_data()
            .map(|data| NativeTemplateFirstOption { data, issuer }))
    }

    pub fn first_option_value(
        &self,
        owner: &NativeTemplateClosedSelect<'_, '_, 'a>,
        option: &NativeTemplateFirstOption<'_, '_, 'a>,
    ) -> Result<NativeTemplateOptionValue<'a>, NativeTemplateLintRefusal> {
        self.first_option_opening_range(owner, option)?;
        Ok(option.data.value)
    }

    /// Equal spans/values, another select in this component or another
    /// callback's token cannot replace this actual stored first header.
    pub fn first_option_opening_range(
        &self,
        owner: &NativeTemplateClosedSelect<'_, '_, 'a>,
        option: &NativeTemplateFirstOption<'_, '_, 'a>,
    ) -> Result<Span, NativeTemplateLintRefusal> {
        let range = self.select_opening_range(owner)?;
        if option.issuer != self.callback_issuer(range)?
            || !core::ptr::eq(option.data.component, self.root.component())
            || !core::ptr::eq(option.data.select, owner.element.original().surface())
            || owner
                .element
                .first_option_data()
                .is_none_or(|data| !core::ptr::eq(data, option.data))
        {
            return Err(NativeTemplateLintRefusal::SourceMismatch);
        }
        Ok(option.data.opening)
    }

    pub fn warn_first_option_with_help(
        &mut self,
        owner: &NativeTemplateClosedSelect<'_, '_, 'a>,
        option: &NativeTemplateFirstOption<'_, '_, 'a>,
        message_key: &str,
        help_key: &str,
    ) -> Result<(), NativeTemplateLintRefusal> {
        let range = self.first_option_opening_range(owner, option)?;
        self.warn_with_help(range, message_key, &[], help_key);
        Ok(())
    }

    fn callback_issuer(&self, range: Span) -> Result<(usize, usize), NativeTemplateLintRefusal> {
        let ordinal = self
            .current_callback
            .ok_or(NativeTemplateLintRefusal::UnsupportedContext { span: range })?;
        // The private ordinal comes from the frozen actual enabled callback
        // vector. Linter identity prevents reuse across distinct registries.
        Ok((core::ptr::from_ref(self.linter).addr(), ordinal))
    }

    fn warn_with_help(
        &mut self,
        range: Span,
        message_key: &str,
        variables: &[(&str, &str)],
        help_key: &str,
    ) {
        self.report_with_help(Severity::Warning, range, message_key, variables, help_key);
    }

    fn report_with_help(
        &mut self,
        default_severity: Severity,
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
        diagnostic.severity = default_severity;
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
