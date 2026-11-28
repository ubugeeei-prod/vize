//! Read retained canonical regions before another consumer takes them.
use super::{syntax, unsupported};
use crate::{
    diagnostic::Severity,
    linter::{LintResult, Linter},
    markup::{L2Markup, MarkupRules},
};
use vize_l0::ToCompactString;
use vize_l1_to_l2::lower::tsrx::{PreparedTsrx, RegionLowering};

impl Linter {
    /// Run this linter's configured TSRX markup rules over an existing lowering.
    /// The source and arena are taken from the producer that owns the regions.
    pub fn lint_prepared_tsrx(&self, prepared: &PreparedTsrx<'_>, filename: &str) -> LintResult {
        let module = prepared.module();
        let mut diagnostics: Vec<_> = module.diagnostics().iter().map(syntax).collect();
        match prepared.regions() {
            RegionLowering::Syntax => {}
            RegionLowering::Refused(issues) => diagnostics.extend(issues.iter().map(unsupported)),
            RegionLowering::Ready(artifacts) => {
                let rules = self.markup_rules(usize::MAX, |_, _| true);
                for region in artifacts.regions() {
                    let result = self.lint_markup_l2(
                        prepared.arena(),
                        module.program().source_text,
                        filename,
                        &L2Markup::from_tsrx_artifact(region.artifact()),
                        None,
                        MarkupRules::new(&rules),
                    );
                    diagnostics.extend(result.diagnostics);
                }
            }
        }
        // The original rule contract keeps the first report on equal nested-region sites.
        diagnostics.sort_by_key(|d| (d.start, d.end, d.rule_name));
        diagnostics.dedup_by_key(|d| (d.start, d.end, d.rule_name));
        let error_count = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        self.apply_diagnostic_policy(LintResult {
            filename: filename.to_compact_string(),
            warning_count: diagnostics.len() - error_count,
            error_count,
            diagnostics,
        })
    }
}
