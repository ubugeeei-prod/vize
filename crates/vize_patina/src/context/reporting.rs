//! Diagnostic reporting shared by template-local and SFC-absolute ranges.

use crate::diagnostic::{LintDiagnostic, Severity};
use vize_atelier_sfc::SfcDescriptor;
use vize_l0::directive::DirectiveSeverity;

use super::eslint_directive::{InlineSuppressionState, parse_eslint_disable_comment};
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
        self.source_offset = vize_l0::SourceRoot::new(descriptor.source.as_ref())
            .ok()
            .and_then(|root| root.whole_block().offset_of(self.source))
            .unwrap_or_else(|| {
                descriptor
                    .template
                    .as_ref()
                    .map_or(0, |template| template.loc.start as u32)
            });
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

    /// A late template capability already addresses the complete SFC. Reuse
    /// the original template pragma state without shifting its ranges twice.
    pub(crate) fn report_template_in_sfc(&mut self, diagnostic: LintDiagnostic) {
        let local = diagnostic
            .start
            .checked_sub(self.source_offset)
            .zip(diagnostic.end.checked_sub(self.source_offset))
            .filter(|(start, end)| start <= end && (*end as usize) <= self.source.len());
        let Some((start, _)) = local else {
            debug_assert!(
                false,
                "a template capability must report within its template frame"
            );
            return;
        };
        self.report_at_line(
            diagnostic,
            self.offset_to_line(start),
            DirectiveDomain::Template,
            0,
        );
    }

    /// Report a diagnostic about a directive comment itself. Directives and
    /// rule selection do not apply to it.
    pub(crate) fn report_directive(&mut self, mut diagnostic: LintDiagnostic) {
        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
        }
        offset_diagnostic(&mut diagnostic, self.source_offset);
        self.diagnostics.push(diagnostic);
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
    /// them against the L0 frame the Davinci stages key spans on, and are
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

/// Drop diagnostics that an `eslint-*` / `oxlint-*` comment in a script or style
/// block already covers. Template comments are applied when the template rule
/// reports; script, CSS, and type-aware diagnostics are appended later and
/// would otherwise ignore the same comments.
pub(crate) fn retain_unless_sfc_suppressed(
    descriptor: &SfcDescriptor<'_>,
    diagnostics: &mut Vec<LintDiagnostic>,
) -> bool {
    let Some(directives) = SfcDirectiveState::scan_if_present(descriptor) else {
        return false;
    };
    let before = diagnostics.len();
    diagnostics.retain(|diagnostic| {
        let line = directives.offset_to_line(diagnostic.start);
        !directives.is_disabled_at(diagnostic.rule_name, line, diagnostic.start)
    });
    diagnostics.len() != before
}

/// Drop diagnostics covered by an `eslint-*` / `oxlint-*` comment in a plain
/// script module. SFC blocks use [`retain_unless_sfc_suppressed`]; standalone
/// `.js` and `.ts` files never enter that scan.
pub(crate) fn retain_unless_source_suppressed(source: &str, diagnostics: &mut Vec<LintDiagnostic>) {
    if diagnostics.is_empty()
        || (!source.contains("eslint-disable")
            && !source.contains("oxlint-disable")
            && !source.contains("eslint-enable")
            && !source.contains("oxlint-enable"))
    {
        return;
    }
    let mut state = InlineSuppressionState::default();
    for (index, line) in source.lines().enumerate() {
        if let Some(directive) = parse_eslint_disable_comment(line) {
            let line_number = u32::try_from(index).unwrap_or(u32::MAX).saturating_add(1);
            state.record(directive, line_number);
        }
    }
    diagnostics.retain(|diagnostic| {
        let line = source_line_number(source, diagnostic.start);
        !state.is_disabled_at(diagnostic.rule_name, line)
    });
}

fn source_line_number(source: &str, start: u32) -> u32 {
    let end = usize::try_from(start).unwrap_or(source.len());
    let prefix = source.get(..end).unwrap_or(source);
    u32::try_from(prefix.bytes().filter(|byte| *byte == b'\n').count()).unwrap_or(u32::MAX) + 1
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
