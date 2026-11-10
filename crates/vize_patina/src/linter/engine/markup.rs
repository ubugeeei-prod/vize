//! The input-neutral markup entry: an L2 artifact, every markup rule in one
//! [`MarkupRules`] walk. JSX/TSX render roots and TSRX components share it.

use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::linter::config::{LintResult, Linter};
use crate::markup::{L2Markup, MarkupContext, MarkupDocument, MarkupRules};
use vize_croquis::Croquis;
use vize_l0::{Allocator, ToCompactString, profile};
use vize_relief::RootNode;

impl Linter {
    /// Lint one L2 markup artifact (a JSX/TSX render root, a TSRX component)
    /// with every enabled markup rule. Spans address `source`.
    pub fn lint_markup(&self, source: &str, filename: &str, markup: &L2Markup<'_>) -> LintResult {
        let capacity = (source.len() * 4).max(self.initial_capacity);
        let allocator = Allocator::with_capacity(capacity);
        let rules = self.markup_rules(usize::MAX, |_, _| true);
        let result = self.lint_markup_l2(
            &allocator,
            source,
            filename,
            markup,
            None,
            MarkupRules::new(&rules),
        );
        self.apply_diagnostic_policy(result)
    }

    /// One markup walk over an L2 artifact.
    pub(super) fn lint_markup_l2<'a>(
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
        self.lint_markup_document(allocator, source, filename, document, analysis, rules)
    }

    /// One markup walk over a lowered Relief root, when the L2 projection
    /// refused that root.
    pub(super) fn lint_markup_root<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        root: &'a RootNode<'a>,
        analysis: Option<&'a Croquis>,
        rules: MarkupRules<'_>,
    ) -> LintResult {
        let document = MarkupDocument::new(root, TemplateSyntax::Vue);
        self.lint_markup_document(allocator, source, filename, document, analysis, rules)
    }

    fn lint_markup_document<'a>(
        &self,
        allocator: &'a Allocator,
        source: &'a str,
        filename: &'a str,
        mut document: MarkupDocument<'a>,
        analysis: Option<&'a Croquis>,
        rules: MarkupRules<'_>,
    ) -> LintResult {
        let mut ctx = self.markup_context(allocator, source, filename);
        if let Some(analysis) = analysis {
            ctx.set_analysis(analysis);
            document = document.with_analysis(analysis);
        }
        profile!("patina.markup.dispatch", {
            let mut markup_ctx = MarkupContext::new(&mut ctx, &document);
            document.visit_with(&rules, &mut markup_ctx);
        });
        finish_markup(filename, ctx)
    }
}

impl Linter {
    pub(super) fn markup_context<'a>(
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
}

pub(super) fn finish_markup(filename: &str, ctx: LintContext<'_>) -> LintResult {
    let error_count = ctx.error_count();
    let warning_count = ctx.warning_count();
    LintResult {
        filename: filename.to_compact_string(),
        diagnostics: ctx.into_diagnostics(),
        error_count,
        warning_count,
    }
}
