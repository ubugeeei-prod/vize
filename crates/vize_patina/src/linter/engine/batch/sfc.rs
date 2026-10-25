use super::{PreparedLint, SfcWork, Work};
use crate::linter::engine::sfc::{
    empty_lint_result,
    pug::{self, PugRoute},
};
use crate::{
    context::SfcSuppression,
    linter::{LintResult, Linter, native_type_aware},
};
use vize_l0::ToCompactString;
use vize_l1_to_l2::lower::pug::PugBlockView;

pub(super) struct SfcTail {
    level: LintResult,
    musea: LintResult,
    art: LintResult,
    suppressions: Option<SfcSuppression>,
    pug: Option<PugBlockView>,
}

impl SfcTail {
    pub(super) fn finish(self, linter: &Linter, mut result: LintResult) -> LintResult {
        super::super::super::severity::apply_severity_overrides(
            &mut result,
            &linter.severity_overrides,
        );
        result = Linter::merge_lint_results(result, self.level);
        result.error_count += self.musea.error_count;
        result.warning_count += self.musea.warning_count;
        result.diagnostics.extend(self.musea.diagnostics);
        result = Linter::merge_lint_results(result, self.art);
        if self
            .suppressions
            .as_ref()
            .is_some_and(|state| state.retain(&mut result.diagnostics))
        {
            result.error_count = result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.severity == crate::Severity::Error)
                .count();
            result.warning_count = result
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.severity == crate::Severity::Warning)
                .count();
        }
        if let Some(view) = self.pug {
            result = pug::map_result(&view, result);
        }
        linter.apply_diagnostic_policy(result)
    }
}

impl Linter {
    /// Prepare a complete SFC while retaining only owned query and finish data.
    pub fn prepare_sfc(&self, source: &str, filename: &str) -> PreparedLint {
        if !native_type_aware::has_active_type_aware_rules(self) {
            return PreparedLint::complete(self.lint_sfc(source, filename));
        }
        match self.route_pug_sfc(source, filename) {
            Some(PugRoute::Complete(result)) => {
                PreparedLint::complete(self.apply_diagnostic_policy(result))
            }
            Some(PugRoute::View(view)) => self
                .prepare_sfc_view(&view.source, filename, true)
                .with_pug(view),
            None => self.prepare_sfc_view(source, filename, false),
        }
    }

    fn prepare_sfc_view(&self, source: &str, filename: &str, derived: bool) -> PreparedLint {
        let descriptor =
            match super::super::super::script_rules::parse_sfc_for_lint(source, filename) {
                Ok(descriptor) => descriptor,
                Err(error) => {
                    let level = Self::sfc_parse_lint_result(filename, source.len(), &error);
                    let template = native_type_aware::lint_sfc_with_corsa_descriptor(
                        self, source, filename, None, derived,
                    );
                    let result = self.append_sfc_document_rule_diagnostics(
                        source,
                        filename,
                        Self::merge_lint_results(template, level),
                        None,
                    );
                    return PreparedLint::complete(self.apply_diagnostic_policy(result));
                }
            };
        let level = self.lint_sfc_level(source, filename, Some(&descriptor));
        let plan = native_type_aware::prepare_with_descriptor(
            self,
            source,
            filename,
            &descriptor,
            derived,
        );
        let mut musea = empty_lint_result(filename);
        super::super::super::musea_rules::append_builtin_musea_diagnostics(
            self, source, filename, &mut musea,
        );
        let mut art = empty_lint_result(filename);
        self.append_art_variant_template_diagnostics(source, filename, &mut art, Some(&descriptor));
        let suppressions = (source.contains("eslint-") || source.contains("oxlint-"))
            .then(|| SfcSuppression::new(&descriptor))
            .flatten();
        PreparedLint {
            filename: filename.to_compact_string(),
            work: Work::Sfc(Box::new(SfcWork {
                plan,
                tail: SfcTail {
                    level,
                    musea,
                    art,
                    suppressions,
                    pug: None,
                },
            })),
        }
    }
}

impl PreparedLint {
    fn with_pug(mut self, view: PugBlockView) -> Self {
        match &mut self.work {
            Work::Sfc(work) => work.tail.pug = Some(view),
            Work::Complete(result) => {
                *result = pug::map_result(
                    &view,
                    std::mem::replace(result, empty_lint_result(&self.filename)),
                );
            }
            Work::Plain { .. } => {}
        }
        self
    }
}
