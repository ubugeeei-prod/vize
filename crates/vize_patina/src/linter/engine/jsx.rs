//! JSX/TSX markup dispatch (Davinci P4-7b).
//!
//! Markup rules run over the P2-16 L2 projection of each render root through
//! the same one-walk [`MarkupRules`] batch as SFC templates. Rules with no
//! markup entry point still walk the lowered Relief root.

use crate::context::LintContext;
use crate::diagnostic::LintDiagnostic;
use crate::ir::TemplateSyntax;
use crate::linter::config::{LintResult, Linter};
use crate::markup::{L2Markup, MarkupContext, MarkupDocument, MarkupRules};
use crate::visitor::LintVisitor;
use vize_croquis::Croquis;
use vize_l0::dialect::VueDialect;
use vize_l0::{Allocator, ToCompactString, profile};
use vize_relief::RootNode;

impl Linter {
    /// Script rules on the JSX program, the same registry `<script>` uses.
    pub(super) fn lint_jsx_script(&self, source: &str, _filename: &str, result: &mut LintResult) {
        super::super::script_rules::append_builtin_script_rules_for_source(self, source, 0, result);
    }

    pub(super) fn jsx_ir_needs_analysis(&self) -> bool {
        self.has_active_semantic_template_rules()
    }

    /// One markup walk over an admitted L2 projection of a render root.
    pub(super) fn lint_jsx_markup_l2<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        markup: &L2Markup<'_>,
        analysis: Option<&'a Croquis>,
        rules: MarkupRules<'_>,
    ) -> LintResult {
        let markup = crate::markup::reborrow_markup(markup);
        let document = MarkupDocument::from_l2(markup, TemplateSyntax::Vue);
        self.lint_jsx_markup(allocator, source, filename, document, analysis, rules)
    }

    /// One markup walk over the lowered Relief root, when the L2 projection
    /// refused that root.
    pub(super) fn lint_jsx_markup_root<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        root: &'a RootNode<'a>,
        analysis: Option<&'a Croquis>,
        rules: MarkupRules<'_>,
    ) -> LintResult {
        let document = MarkupDocument::new(root, TemplateSyntax::Vue);
        self.lint_jsx_markup(allocator, source, filename, document, analysis, rules)
    }

    fn lint_jsx_markup<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        mut document: MarkupDocument<'a>,
        analysis: Option<&'a Croquis>,
        rules: MarkupRules<'_>,
    ) -> LintResult {
        let mut ctx = self.jsx_context(allocator, source, filename);
        if let Some(analysis) = analysis {
            ctx.set_analysis(analysis);
            document = document.with_analysis(analysis);
        }
        profile!("patina.jsx.markup.visit", {
            let mut markup_ctx = MarkupContext::new(&mut ctx, &document);
            document.visit_with(&rules, &mut markup_ctx);
        });
        finish_jsx(filename, ctx)
    }

    /// Legacy rules (no markup entry point) over one lowered Relief root.
    pub(super) fn lint_jsx_fallback_root(
        &self,
        allocator: &Allocator,
        source: &str,
        filename: &str,
        root: &RootNode<'_>,
        keep_mask: &[bool],
    ) -> LintResult {
        let mut ctx = self.jsx_context(allocator, source, filename);
        ctx.set_dialect(VueDialect::Vue);
        // The visitor zips rules with their names and reads the mask with
        // `get`, so unequal lengths cannot misalign or panic.
        let mut visitor = LintVisitor::with_rule_filter(
            &mut ctx,
            self.registry.rules(),
            self.rule_names(),
            self.registry.has_exit_element_rules(),
            keep_mask,
        );
        profile!("patina.jsx.fallback.visit", visitor.visit_root(root));
        finish_jsx(filename, ctx)
    }

    fn jsx_context<'a>(
        &'a self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
    ) -> LintContext<'a> {
        let mut ctx = LintContext::with_locale(allocator, source, filename, self.locale);
        ctx.set_enabled_rules(self.enabled_rules.clone());
        ctx.set_config_disabled_rules(self.disabled_rules.clone());
        ctx.set_config_rule_severities(self.severity_overrides.clone());
        ctx.set_help_level(self.help_level);
        ctx.set_custom_elements(&self.custom_elements);
        ctx
    }

    /// Drop diagnostics `result` already reported in `seen` (rule name and range).
    pub(super) fn dedupe_against(result: &mut LintResult, seen: &LintResult) {
        if seen.diagnostics.is_empty() || result.diagnostics.is_empty() {
            return;
        }
        result.diagnostics.retain(|candidate| {
            !seen.diagnostics.iter().any(|existing| {
                existing.rule_name == candidate.rule_name
                    && existing.start == candidate.start
                    && existing.end == candidate.end
            })
        });
        result.error_count = result
            .diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic.severity, crate::diagnostic::Severity::Error))
            .count();
        result.warning_count = result.diagnostics.len() - result.error_count;
    }

    pub(super) fn jsx_diagnostics_lint_result(
        filename: &str,
        diagnostics: &[vize_atelier_jsx::JsxDiagnostic],
    ) -> LintResult {
        const JSX_PARSE_RULE: &str = "parser/jsx";
        let mut lint_diagnostics = Vec::with_capacity(diagnostics.len());
        let mut error_count = 0;
        let mut warning_count = 0;
        for diagnostic in diagnostics {
            let lint_diagnostic = if diagnostic.is_error() {
                error_count += 1;
                LintDiagnostic::error(
                    JSX_PARSE_RULE,
                    diagnostic.message.clone(),
                    diagnostic.start,
                    diagnostic.end,
                )
            } else {
                warning_count += 1;
                LintDiagnostic::warn(
                    JSX_PARSE_RULE,
                    diagnostic.message.clone(),
                    diagnostic.start,
                    diagnostic.end,
                )
            };
            lint_diagnostics.push(lint_diagnostic);
        }
        LintResult {
            filename: filename.to_compact_string(),
            diagnostics: lint_diagnostics,
            error_count,
            warning_count,
        }
    }
}

fn finish_jsx(filename: &str, ctx: LintContext<'_>) -> LintResult {
    let error_count = ctx.error_count();
    let warning_count = ctx.warning_count();
    LintResult {
        filename: filename.to_compact_string(),
        diagnostics: ctx.into_diagnostics(),
        error_count,
        warning_count,
    }
}
