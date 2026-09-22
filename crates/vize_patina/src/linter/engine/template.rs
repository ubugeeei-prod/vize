//! The template rule lanes (Davinci P4-7b).
//!
//! A template runs through two lanes over one [`LintContext`]:
//!
//! 1. the directive visitor ([`LintVisitor`]), for the rules that have no
//!    markup body yet — it also registers every suppression comment
//!    (`@vize:forget`, ignore regions, severity levels) before any rule of
//!    either lane reports;
//! 2. the markup lane: every markup-capable rule's [`crate::markup::MarkupRule`]
//!    body, fused into one walk of the markup facade, each rule called only
//!    at the hooks it subscribes to ([`crate::markup::MarkupHooks`]).
//!
//! A rule runs in exactly one lane. Both lanes read the one lint parse: the
//! facade's S2 backend replaces it when the directive lane empties and the
//! Relief parse retires (lowering a second tree per template now would double
//! the parse cost). The switch oracle (`markup::differential::switch`) proves
//! each markup body reports exactly what its directive hooks reported, over
//! both backends.

use super::super::config::{LintResult, Linter};
use super::TemplateRuleEnv;
use super::lane_plan::LanePlan;
use crate::context::LintContext;
use crate::diagnostic::LintDiagnostic;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument};
use crate::visitor::LintVisitor;
use vize_croquis::Croquis;
use vize_relief::RootNode;
use vize_s0::{Allocator, ToCompactString, profile};

impl Linter {
    pub(super) fn template_rule_count_for_source(
        &self,
        template_source: &str,
        sfc_source: Option<&str>,
    ) -> usize {
        if !matches!(self.preset, Some(crate::preset::LintPreset::Ecosystem))
            || self.enabled_rules.is_some()
            || !self.disabled_rules.is_empty()
            || super::ecosystem_hint::source_may_contain_ecosystem_template_rule(
                template_source,
                sfc_source,
            )
        {
            return self.registry.rules().len();
        }

        self.registry
            .rules()
            .len()
            .saturating_sub(crate::rules::ecosystem::TEMPLATE_RULE_COUNT)
    }

    pub(super) fn run_template_rules<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        root: &'a RootNode<'a>,
        analysis: Option<&'a Croquis>,
        env: TemplateRuleEnv<'a>,
    ) -> LintResult {
        let rule_count = self.template_rule_count_for_source(
            source,
            env.sfc_descriptor
                .map(|descriptor| descriptor.source.as_ref()),
        );
        let rules = &self.registry.rules()[..rule_count];
        // A rule runs in exactly one lane: its markup body when it has one,
        // its directive hooks otherwise.
        let names = &self.rule_names()[..rule_count];
        let plan = self.lane_plan.get_or_init(|| LanePlan::new(&self.registry));
        let markup_rules = plan.markup(rules, names);

        let mut ctx = LintContext::with_locale(allocator, source, filename, self.locale);
        ctx.set_enabled_rules(self.enabled_rules.clone());
        ctx.set_config_disabled_rules(self.disabled_rules.clone());
        ctx.set_config_rule_severities(self.severity_overrides.clone());
        ctx.set_help_level(self.help_level);
        ctx.set_dialect(env.dialect);
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

        // Lane 1: the directive visitor, for the rules without a facade.
        let mut visitor = LintVisitor::with_active_rules(
            &mut ctx,
            plan.directive(rules, names),
            self.registry.has_exit_element_rules(),
        );
        profile!("patina.template.visit", visitor.visit_root(root));
        let directive_reports = ctx.diagnostics().len();

        // Lane 2: the markup rules' bodies, fused into one facade walk.
        if !markup_rules.is_empty() {
            let mut document = MarkupDocument::new(root, TemplateSyntax::Vue);
            if let Some(analysis) = analysis {
                document = document.with_analysis(analysis);
            }
            profile!("patina.template.markup.visit", {
                let mut markup_ctx = MarkupContext::new(&mut ctx, &document);
                document.visit_rules(&markup_rules, &mut markup_ctx);
            });
        }

        let error_count = ctx.error_count();
        let warning_count = ctx.warning_count();
        let diagnostics = merge_lanes(ctx.into_diagnostics(), directive_reports);

        LintResult {
            filename: filename.to_compact_string(),
            diagnostics,
            error_count,
            warning_count,
        }
    }
}

/// Interleave the two lanes' reports in source order. Each lane reports in its
/// own walk order; merging on the start offset (the directive lane first on a
/// tie) restores the one-walk order a template reported in before the switch,
/// without reordering either lane.
fn merge_lanes(
    diagnostics: std::vec::Vec<LintDiagnostic>,
    directive_reports: usize,
) -> std::vec::Vec<LintDiagnostic> {
    let (directive, markup) = diagnostics.split_at(directive_reports);
    let in_order = match (
        directive.iter().map(|report| report.start).max(),
        markup.first(),
    ) {
        (Some(last), Some(first)) => last <= first.start,
        _ => true,
    };
    if in_order {
        return diagnostics;
    }
    let mut merged = std::vec::Vec::with_capacity(diagnostics.len());
    let mut all = diagnostics.into_iter();
    let directive: std::vec::Vec<LintDiagnostic> = all.by_ref().take(directive_reports).collect();
    let mut directive = directive.into_iter().peekable();
    let mut markup = all.peekable();
    while let (Some(left), Some(right)) = (directive.peek(), markup.peek()) {
        let next = if right.start < left.start {
            markup.next()
        } else {
            directive.next()
        };
        merged.extend(next);
    }
    merged.extend(directive);
    merged.extend(markup);
    merged
}
