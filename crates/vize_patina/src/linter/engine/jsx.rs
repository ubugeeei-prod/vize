//! JSX/TSX markup dispatch (Davinci P4-7b).
//!
//! Markup rules run over the P2-16 S2 projection of each render root. Rules
//! with no markup entry point still walk the lowered Relief root.

use crate::context::LintContext;
use crate::diagnostic::LintDiagnostic;
use crate::ir::TemplateSyntax;
use crate::linter::config::{LintResult, Linter};
use crate::markup::{MarkupContext, MarkupDocument, S2Markup};
use crate::visitor::LintVisitor;
use vize_croquis::Croquis;
use vize_relief::RootNode;
use vize_s0::dialect::VueDialect;
use vize_s0::{Allocator, ToCompactString, profile};

impl Linter {
    /// Script rules on the JSX program, the same registry `<script>` uses.
    pub(super) fn lint_jsx_script(&self, source: &str, result: &mut LintResult) {
        super::super::script_rules::append_builtin_script_rules_for_source(self, source, 0, result);
    }

    pub(super) fn jsx_ir_needs_analysis(&self) -> bool {
        self.has_active_semantic_template_rules()
    }

    /// Markup rules whose JSX shape is on the S2 projection without a list or
    /// branch (`jsx_needs_lowering` is false).
    pub(super) fn lint_jsx_over_ir<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        markup: &S2Markup<'_>,
        analysis: Option<&'a Croquis>,
    ) -> LintResult {
        self.visit_jsx_s2(allocator, source, filename, markup, analysis, false)
    }

    /// List/branch markup rules over an admitted S2 projection.
    pub(super) fn lint_jsx_lowered_markup_s2<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        markup: &S2Markup<'_>,
    ) -> LintResult {
        self.visit_jsx_s2(allocator, source, filename, markup, None, true)
    }

    /// List/branch markup rules (`jsx_needs_lowering`) over the lowered Relief
    /// root, when the S2 projection refused that root.
    pub(super) fn lint_jsx_lowered_markup_root<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        root: &RootNode<'a>,
    ) -> LintResult {
        let mut ctx = self.jsx_context(allocator, source, filename);
        let document = MarkupDocument::new(root, TemplateSyntax::Vue);
        profile!("patina.jsx.lowered_markup.visit", {
            let mut markup_ctx = MarkupContext::new(&mut ctx, &document);
            for rule in self.registry.rules() {
                if rule.jsx_needs_lowering()
                    && let Some(markup_rule) = rule.as_markup_rule()
                {
                    document.visit_with(markup_rule, &mut markup_ctx);
                }
            }
        });
        finish_jsx(filename, ctx)
    }

    /// Element-shaped markup rules when the P2-16 projection refuses the root.
    /// Same rule partition as [`Self::lint_jsx_over_ir`], over the lowered root.
    pub(super) fn lint_jsx_over_ir_refused<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        root: &RootNode<'a>,
        analysis: Option<&'a Croquis>,
    ) -> LintResult {
        let mut ctx = self.jsx_context(allocator, source, filename);
        let mut document = MarkupDocument::new(root, TemplateSyntax::Vue);
        if let Some(analysis) = analysis {
            ctx.set_analysis(analysis);
            document = document.with_analysis(analysis);
        }
        profile!("patina.jsx.ir.visit", {
            let mut markup_ctx = MarkupContext::new(&mut ctx, &document);
            for rule in self.registry.rules() {
                if rule.jsx_needs_lowering() {
                    continue;
                }
                if let Some(markup_rule) = rule.as_markup_rule() {
                    document.visit_with(markup_rule, &mut markup_ctx);
                }
            }
        });
        finish_jsx(filename, ctx)
    }

    fn visit_jsx_s2<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        markup: &S2Markup<'_>,
        analysis: Option<&'a Croquis>,
        needs_lowering: bool,
    ) -> LintResult {
        let markup = crate::markup::reborrow_markup(markup);
        let mut ctx = self.jsx_context(allocator, source, filename);
        let mut document = MarkupDocument::from_s2(markup, TemplateSyntax::Vue);
        if let Some(analysis) = analysis {
            ctx.set_analysis(analysis);
            document = document.with_analysis(analysis);
        }
        profile!("patina.jsx.ir.visit", {
            let mut markup_ctx = MarkupContext::new(&mut ctx, &document);
            for rule in self.registry.rules() {
                if rule.jsx_needs_lowering() != needs_lowering {
                    continue;
                }
                if let Some(markup_rule) = rule.as_markup_rule() {
                    document.visit_with(markup_rule, &mut markup_ctx);
                }
            }
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
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
    ) -> LintContext<'a> {
        let mut ctx = LintContext::with_locale(allocator, source, filename, self.locale);
        ctx.set_enabled_rules(self.enabled_rules.clone());
        ctx.set_config_disabled_rules(self.disabled_rules.clone());
        ctx.set_config_rule_severities(self.severity_overrides.clone());
        ctx.set_help_level(self.help_level);
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
