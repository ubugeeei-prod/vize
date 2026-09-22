//! One SFC markup rule on the S2 facade (Davinci P4-7b).
//!
//! Every other template rule still walks the Relief visitor. [`RULE`] is the
//! markup rule whose SFC diagnostics already match that visitor, so `lint_sfc`
//! drives it here instead.

use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::linter::config::Linter;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule, S2Template};
use crate::visitor::LintVisitor;
use vize_croquis::Croquis;
use vize_relief::RootNode;
use vize_s0::{Allocator, profile};

/// `vue/no-multi-spaces` — opening-tag gap spans match the Relief visitor.
pub(in crate::linter::engine) const RULE: &str = "vue/no-multi-spaces";

pub(in crate::linter::engine) struct Dispatch<'a> {
    pub allocator: &'a Allocator,
    pub source: &'a str,
    pub root: &'a RootNode<'a>,
    pub analysis: Option<&'a Croquis>,
    pub rule: Option<&'static str>,
    pub rule_count: usize,
}

pub(in crate::linter::engine) fn dispatch_template_rules<'a>(
    linter: &Linter,
    ctx: &mut LintContext<'a>,
    input: Dispatch<'a>,
) {
    let rules = &linter.registry.rules()[..input.rule_count];
    let names = &linter.rule_names()[..input.rule_count];
    let exit = linter.registry.has_exit_element_rules();
    let facade_index = input.rule.and_then(|name| {
        let index = names.iter().position(|candidate| *candidate == name)?;
        rules[index].as_markup_rule()?;
        Some(index)
    });

    let Some(index) = facade_index else {
        let mut visitor = LintVisitor::new(ctx, rules, names, exit);
        profile!("patina.template.visit", visitor.visit_root(input.root));
        return;
    };

    let mut keep = vec![true; input.rule_count];
    keep[index] = false;
    {
        let mut visitor = LintVisitor::with_rule_filter(ctx, rules, names, exit, &keep);
        profile!("patina.template.visit", visitor.visit_root(input.root));
    }
    if let Some(rule) = rules[index].as_markup_rule() {
        visit_facade(ctx, &input, rule);
    }
}

fn visit_facade<'a>(ctx: &mut LintContext<'a>, input: &Dispatch<'a>, rule: &dyn MarkupRule) {
    let lowered = S2Template::lower(input.allocator, input.source);
    let markup = lowered.markup();
    let markup = crate::markup::reborrow_markup(&markup);
    let mut document = MarkupDocument::from_s2(markup, TemplateSyntax::Vue);
    if let Some(analysis) = input.analysis {
        document = document.with_analysis(analysis);
    }
    profile!("patina.sfc.facade.visit", {
        let mut markup_ctx = MarkupContext::new(ctx, &document);
        document.visit_with(rule, &mut markup_ctx);
    });
}
