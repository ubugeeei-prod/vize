//! Template execution and finalization share one directive context.

use super::{
    Allocator, LintResult, Linter, RootNode, TemplateAnalysis, TemplateRuleEnv, sfc,
    unused_directives,
};
use crate::context::LintContext;
use vize_croquis::{Croquis, Drawer};
use vize_l0::{ToCompactString, profile};

pub(super) struct TemplateExecution<'a> {
    pub allocator: &'a Allocator,
    pub source: &'a str,
    pub filename: &'a str,
    pub root: &'a RootNode<'a>,
    pub env: TemplateRuleEnv<'a>,
}

impl Linter {
    fn run_template_rules<'a>(
        &'a self,
        input: TemplateExecution<'a>,
        analysis: Option<&'a Croquis>,
        finish: impl FnOnce(&mut LintContext<'_>),
    ) -> LintResult {
        let TemplateExecution {
            allocator,
            source,
            filename,
            root,
            env,
        } = input;
        let mut ctx = LintContext::with_locale(allocator, source, filename, self.locale);
        ctx.set_enabled_rules(self.enabled_rules.clone());
        ctx.set_config_disabled_rules(self.disabled_rules.clone());
        ctx.set_config_rule_severities(self.severity_overrides.clone());
        ctx.set_help_level(self.help_level);
        ctx.set_custom_elements(&self.custom_elements);
        ctx.set_dialect(env.dialect);
        ctx.art_script_analysis = env.art_script_analysis;
        if let Some(descriptor) = env.sfc_descriptor {
            ctx.set_sfc_template_descriptor(descriptor);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let has_analysis = analysis.is_some();
        if let Some(analysis) = analysis {
            ctx.set_analysis(analysis);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if has_analysis && super::super::native_type_aware::has_active_type_aware_rules(self) {
            ctx.set_analysis_excluded_rules(super::super::native_type_aware::TYPE_AWARE_RULES);
        }
        let rule_count = self.template_rule_count_for_source(
            source,
            env.sfc_descriptor
                .map(|descriptor| descriptor.source.as_ref()),
        );
        sfc::facade::dispatch_template_rules(
            self,
            &mut ctx,
            sfc::facade::Dispatch {
                allocator,
                source,
                root,
                analysis,
                rules: env.facade_rules,
                rule_count,
            },
        );
        finish(&mut ctx);
        if self.report_unused_disable_directives {
            unused_directives::report(self, &mut ctx);
        }

        let error_count = ctx.error_count();
        let warning_count = ctx.warning_count();
        let diagnostics = ctx.into_diagnostics();

        LintResult {
            filename: filename.to_compact_string(),
            diagnostics,
            error_count,
            warning_count,
        }
    }

    pub(super) fn lint_template_root<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        root: &RootNode<'a>,
        analysis: TemplateAnalysis<'a>,
        env: TemplateRuleEnv<'a>,
    ) -> LintResult {
        self.lint_template_root_with_finish(
            TemplateExecution {
                allocator,
                source,
                filename,
                root,
                env,
            },
            analysis,
            |_| {},
        )
    }

    /// Late template producers report before the same pragma state is finalized.
    pub(super) fn lint_template_root_with_finish<'a>(
        &self,
        input: TemplateExecution<'a>,
        analysis: TemplateAnalysis<'a>,
        finish: impl FnOnce(&mut LintContext<'_>),
    ) -> LintResult {
        let owned_analysis;
        let analysis = match analysis {
            TemplateAnalysis::Disabled => None,
            _ if !self.has_active_semantic_template_rules() => None,
            TemplateAnalysis::Precomputed(analysis) => Some(analysis),
            TemplateAnalysis::Lazy => {
                owned_analysis = profile!("patina.template.croquis", {
                    let mut analyzer = Drawer::for_lint();
                    analyzer.analyze_template(input.root);
                    analyzer.finish()
                });
                Some(&owned_analysis)
            }
        };

        self.run_template_rules(input, analysis, finish)
    }
}
