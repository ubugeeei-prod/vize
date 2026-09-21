//! Diagnostic reporting shared by template-local and SFC-absolute ranges.

use crate::diagnostic::{LintDiagnostic, Severity};
use vize_atelier_sfc::SfcDescriptor;
use vize_s0::directive::DirectiveSeverity;

use super::{LintContext, SfcDirectiveState};

#[derive(Clone, Copy)]
enum DirectiveDomain {
    Template,
    Sfc,
}

impl<'a> LintContext<'a> {
    /// Attach an SFC descriptor to a template context and prepare its absolute
    /// source position. Full-document directives remain lazy until needed.
    pub fn set_sfc_template_descriptor(&mut self, descriptor: &'a SfcDescriptor<'a>) {
        self.source_offset = descriptor
            .template
            .as_ref()
            .map_or(0, |template| template.loc.start as u32);
        self.sfc_directives = None;
        self.sfc_directives_scanned = false;
        self.set_sfc_descriptor(descriptor);
    }

    /// Report a diagnostic whose range addresses the context's local source.
    #[inline]
    pub fn report(&mut self, diagnostic: LintDiagnostic) {
        let line = self.offset_to_line(diagnostic.start);
        self.report_at_line(
            diagnostic,
            line,
            DirectiveDomain::Template,
            self.source_offset,
        );
    }

    /// Report a diagnostic whose range already addresses the containing SFC.
    #[inline]
    pub fn report_in_sfc(&mut self, diagnostic: LintDiagnostic) {
        let line = self
            .sfc_directives()
            .map_or(0, |directives| directives.offset_to_line(diagnostic.start));
        self.report_at_line(diagnostic, line, DirectiveDomain::Sfc, 0);
    }

    /// Report a diagnostic whose range addresses croquis' script-analysis
    /// frame — where macro calls and bindings are measured.
    ///
    /// The ranges move into the complete file through
    /// [`ScriptFrame`](crate::output::frame::ScriptFrame), which validates
    /// them against the S0 frame the Davinci stages key spans on, and are
    /// then reported like any SFC-absolute range. This is FP-1's root fix: a
    /// script-analysis range reported through [`LintContext::report`] was
    /// shifted by the *template's* offset and rendered on unrelated lines.
    ///
    /// Without an SFC descriptor there is no script frame, and a range
    /// outside the script blocks cannot be located honestly; neither is
    /// reported, and a debug build treats both as a caller bug.
    pub fn report_in_script(&mut self, diagnostic: LintDiagnostic) {
        let reframed = self
            .sfc_descriptor
            .and_then(crate::output::frame::ScriptFrame::for_lint)
            .and_then(|frame| frame.reframe(&diagnostic));
        let Some(reframed) = reframed else {
            debug_assert!(
                false,
                "{}: a script-analysis range needs an SFC script block that contains it",
                diagnostic.rule_name
            );
            return;
        };
        self.report_in_sfc(reframed);
    }

    fn sfc_directives(&mut self) -> Option<&SfcDirectiveState> {
        if !self.sfc_directives_scanned {
            self.sfc_directives_scanned = true;
            self.sfc_directives = self
                .sfc_descriptor
                .and_then(SfcDirectiveState::scan_if_present);
        }
        self.sfc_directives.as_ref()
    }

    fn report_at_line(
        &mut self,
        mut diagnostic: LintDiagnostic,
        line: u32,
        domain: DirectiveDomain,
        output_offset: u32,
    ) {
        if !self.is_rule_enabled(diagnostic.rule_name) {
            return;
        }

        let (disabled, expected, override_severity) = match domain {
            DirectiveDomain::Template => (
                self.is_disabled_at(diagnostic.rule_name, line),
                self.expected_error_lines.contains(&line),
                self.severity_overrides.get(&line).copied(),
            ),
            // Read through the lazy accessor so a caller that reports an
            // SFC-absolute range without resolving directives first still
            // honours full-document comments.
            DirectiveDomain::Sfc => {
                let rule_name = diagnostic.rule_name;
                self.sfc_directives()
                    .map_or((false, false, None), |directives| {
                        (
                            directives.is_disabled_at(rule_name, line, diagnostic.start),
                            directives.is_expected_at(line, diagnostic.start),
                            directives.severity_at(line, diagnostic.start),
                        )
                    })
            }
        };
        if disabled || expected {
            return;
        }

        if let Some(severity) = self.config_rule_severities.get(diagnostic.rule_name) {
            diagnostic.severity = *severity;
        }
        match override_severity {
            Some(DirectiveSeverity::Off) => return,
            Some(DirectiveSeverity::Warn) => diagnostic.severity = Severity::Warning,
            Some(DirectiveSeverity::Error) => diagnostic.severity = Severity::Error,
            None => {}
        }

        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
        }
        offset_diagnostic(&mut diagnostic, output_offset);
        self.diagnostics.push(diagnostic);
    }
}

/// Shift every range carried by a diagnostic into its containing source.
pub(crate) fn offset_diagnostic(diagnostic: &mut LintDiagnostic, byte_offset: u32) {
    if byte_offset == 0 {
        return;
    }
    diagnostic.start += byte_offset;
    diagnostic.end += byte_offset;
    for label in &mut diagnostic.labels {
        label.start += byte_offset;
        label.end += byte_offset;
    }
    if let Some(fix) = diagnostic.fix.as_mut() {
        for edit in &mut fix.edits {
            edit.start += byte_offset;
            edit.end += byte_offset;
        }
    }
}
