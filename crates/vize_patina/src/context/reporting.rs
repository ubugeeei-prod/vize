//! Diagnostic reporting shared by template-local and SFC-absolute ranges.

use crate::diagnostic::{LintDiagnostic, Severity};
use vize_atelier_sfc::SfcDescriptor;
use vize_l0::{cstr, directive::DirectiveSeverity};

use super::eslint_directive::{SuppressionSource, UnusedDirective};
use super::{LintContext, SfcDirectiveState};

const UNUSED_RULE: &str = "vue/comment-directive";

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
        let directive_offset = diagnostic.start;
        let line = self.offset_to_line(diagnostic.start);
        self.report_at_line(
            diagnostic,
            line,
            DirectiveDomain::Template,
            self.source_offset,
            directive_offset,
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
            start,
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
        self.record_emission();
        self.diagnostics.push(diagnostic);
    }

    /// Report a diagnostic whose range already addresses the containing SFC.
    #[inline]
    pub fn report_in_sfc(&mut self, diagnostic: LintDiagnostic) {
        let directive_offset = diagnostic.start;
        let line = self
            .sfc_directives()
            .map_or(0, |directives| directives.offset_to_line(diagnostic.start));
        self.report_at_line(diagnostic, line, DirectiveDomain::Sfc, 0, directive_offset);
    }

    /// Report a diagnostic whose range addresses croquis' script-analysis
    /// frame — where macro calls and bindings are measured.
    ///
    /// The ranges move into the complete file through
    /// the internal `ScriptFrame`, which validates
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
        directive_offset: u32,
    ) {
        if !self.is_rule_enabled(diagnostic.rule_name) {
            return;
        }

        let (disabled, expected, override_severity) = match domain {
            DirectiveDomain::Template => (
                self.is_rule_disabled_at_offset(diagnostic.rule_name, directive_offset),
                self.expected_error_lines.contains(&line),
                self.severity_overrides.get(&line).copied(),
            ),
            // Read through the lazy accessor so a caller that reports an
            // SFC-absolute range without resolving directives first still
            // honours full-document comments.
            DirectiveDomain::Sfc => {
                let rule_name = diagnostic.rule_name;
                let defer_suppression = self.defer_exact_suppressions;
                self.sfc_directives()
                    .map_or((false, false, None), |directives| {
                        (
                            !defer_suppression
                                && directives.is_disabled_at(rule_name, line, diagnostic.start),
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
        self.record_emission();
        self.diagnostics.push(diagnostic);
    }
}

/// Owned suppression state retained across project snapshot publication.
pub(crate) struct SfcSuppression(SfcDirectiveState);

impl SfcSuppression {
    pub(crate) fn new(descriptor: &SfcDescriptor<'_>) -> Option<Self> {
        SfcDirectiveState::scan_if_present(descriptor).map(Self)
    }

    pub(crate) fn retain(&self, diagnostics: &mut Vec<LintDiagnostic>) -> bool {
        let before = diagnostics.len();
        diagnostics.retain(|diagnostic| {
            let line = self.0.offset_to_line(diagnostic.start);
            !self
                .0
                .is_disabled_at(diagnostic.rule_name, line, diagnostic.start)
        });
        diagnostics.len() != before
    }

    pub(crate) fn append_unused(
        &self,
        diagnostics: &mut Vec<LintDiagnostic>,
        is_active: impl Fn(&str) -> bool,
    ) -> bool {
        let unused = self.0.unused(is_active);
        let changed = !unused.is_empty();
        diagnostics.extend(unused.into_iter().map(unused_diagnostic));
        changed
    }
}

/// Drop diagnostics covered by an `eslint-*`, `oxlint-*`, or `vize-*` comment in a plain
/// script module. SFC blocks use [`retain_unless_sfc_suppressed`]; standalone
/// `.js` and `.ts` files never enter that scan.
pub(crate) fn retain_unless_source_suppressed(
    source: &str,
    filename: &str,
    diagnostics: &mut Vec<LintDiagnostic>,
    report_unused: bool,
    is_active: impl Fn(&str) -> bool,
) {
    if (!report_unused && diagnostics.is_empty())
        || (!source.contains("eslint-disable")
            && !source.contains("oxlint-disable")
            && !source.contains("eslint-enable")
            && !source.contains("oxlint-enable")
            && !source.contains("vize-disable")
            && !source.contains("vize-enable"))
    {
        return;
    }
    let state = super::sfc_directives::script_suppressions(source, filename);
    diagnostics.retain(|diagnostic| {
        let line = source_line_number(source, diagnostic.start);
        !state.is_disabled_at(diagnostic.rule_name, line)
    });
    if report_unused {
        diagnostics.extend(state.unused(is_active).into_iter().map(unused_diagnostic));
    }
}

fn unused_diagnostic(directive: UnusedDirective) -> LintDiagnostic {
    let message = match (directive.source, directive.rule.as_deref()) {
        (SuppressionSource::Eslint, Some(rule)) => {
            cstr!("Unused eslint-disable directive (no problems were reported from '{rule}').")
        }
        (SuppressionSource::Eslint, None) => {
            "Unused eslint-disable directive (no problems were reported).".into()
        }
        (SuppressionSource::VizeForget, Some(rule)) => {
            cstr!("Unused @vize:forget directive (no problems were reported from '{rule}').")
        }
        (SuppressionSource::VizeForget, None) => {
            "Unused @vize:forget directive (no problems were reported).".into()
        }
    };
    LintDiagnostic::error(UNUSED_RULE, message, directive.start, directive.end)
}

fn source_line_number(source: &str, start: u32) -> u32 {
    let end = usize::try_from(start).unwrap_or(source.len());
    let prefix = source.get(..end).unwrap_or(source);
    u32::try_from(prefix.bytes().filter(|byte| *byte == b'\n').count()).unwrap_or(u32::MAX) + 1
}

#[cfg(test)]
mod tests {
    use super::SfcSuppression;
    use crate::{LintContext, LintDiagnostic};
    use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
    use vize_l0::Allocator;

    #[test]
    fn final_sfc_pass_uses_exact_node_identity_for_type_aware_diagnostics() {
        let source = r#"<template>
<!-- @vize:forget type/no-unsafe-template-binding -->
<button :disabled="unsafe" />
</template>"#;
        let descriptor = parse_sfc(
            source,
            SfcParseOptions {
                filename: "TypeAware.vue".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let start = source.find("<button").unwrap() as u32;
        let mut diagnostics = vec![
            LintDiagnostic::error(
                "type/no-unsafe-template-binding",
                "unsafe binding",
                start,
                start + 7,
            ),
            LintDiagnostic::error("vue/html-self-closing", "peer finding", start, start + 7),
        ];
        let suppressions = SfcSuppression::new(&descriptor).unwrap();

        assert!(suppressions.retain(&mut diagnostics));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_name, "vue/html-self-closing");
        assert!(!suppressions.append_unused(&mut diagnostics, |_| true));
    }

    #[test]
    fn final_sfc_pass_owns_region_usage_for_late_diagnostics() {
        let source = r#"<script>
/* eslint-disable script/no-with-defaults */
const value = 1;
</script>
<template><div /></template>"#;
        let descriptor = parse_sfc(
            source,
            SfcParseOptions {
                filename: "LateScriptDiagnostic.vue".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let allocator = Allocator::default();
        let mut context = LintContext::new(
            &allocator,
            descriptor.template.as_ref().unwrap().content.as_ref(),
            "LateScriptDiagnostic.vue",
        );
        context.set_sfc_template_descriptor(&descriptor);
        context.defer_exact_suppressions();
        let start = source.find("const value").unwrap() as u32;
        context.report_in_sfc(LintDiagnostic::error(
            "script/no-with-defaults",
            "late script diagnostic",
            start,
            start + 5,
        ));
        let mut diagnostics = context.into_diagnostics();
        assert_eq!(diagnostics.len(), 1);

        let suppressions = SfcSuppression::new(&descriptor).unwrap();
        assert!(suppressions.retain(&mut diagnostics));
        assert!(diagnostics.is_empty());
        assert!(!suppressions.append_unused(&mut diagnostics, |_| true));
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
