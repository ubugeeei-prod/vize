//! SFC markup rules whose diagnostics match the Relief visitor (P4-7b).
//!
//! The admitted rules share one S2 lowering and one traversal. All other
//! template rules stay on the Relief visitor until their parity is proven.

use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::linter::config::Linter;
use crate::markup::{MarkupContext, MarkupDocument, S2Template};
use crate::visitor::LintVisitor;
use vize_croquis::Croquis;
use vize_relief::RootNode;
use vize_s0::{Allocator, profile};

mod batch;

/// Exact diagnostic, help, label and fix parity is checked by the SFC battery.
pub(in crate::linter::engine) const RULES: &[&str] = &[
    "vue/no-textarea-mustache",
    "vue/no-multi-spaces",
    "vue/no-template-target-blank",
    "vue/no-unsandboxed-iframe",
    "vue/no-invalid-html-attribute",
    "a11y/img-alt",
    "a11y/heading-has-content",
    "a11y/iframe-has-title",
    "a11y/no-distracting-elements",
    "a11y/no-i-for-icon",
    "a11y/tabindex-no-positive",
    "a11y/form-control-has-label",
    "a11y/no-aria-hidden-on-focusable",
    "a11y/no-role-presentation-on-focusable",
    "a11y/mouse-events-have-key-events",
    "a11y/anchor-is-valid",
    "a11y/media-has-caption",
    "html/deprecated-element",
    "html/deprecated-attr",
    "html/no-consecutive-br",
    "html/no-duplicate-dt",
    "html/require-datetime",
    "vue/html-button-has-type",
    "vue/no-inline-style",
    "vue/no-boolean-attr-value",
    "a11y/heading-levels",
    "a11y/placeholder-label-option",
    "a11y/use-list",
    "html/no-duplicate-class",
    "html/no-dupe-style-properties",
];

pub(in crate::linter::engine) struct Dispatch<'a> {
    pub allocator: &'a Allocator,
    pub source: &'a str,
    pub root: &'a RootNode<'a>,
    pub analysis: Option<&'a Croquis>,
    pub rules: &'static [&'static str],
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
    let selected: Vec<_> = rules
        .iter()
        .enumerate()
        .filter_map(|(index, rule)| {
            let name = names[index];
            (input.rules.contains(&name) && linter.is_rule_enabled(name))
                .then(|| rule.as_markup_rule().map(|rule| (index, rule.name(), rule)))
                .flatten()
        })
        .collect();

    if selected.is_empty() {
        let mut visitor = LintVisitor::new(ctx, rules, names, exit);
        profile!("patina.template.visit", visitor.visit_root(input.root));
        return;
    }

    let mut keep = vec![true; input.rule_count];
    for (index, _, _) in &selected {
        keep[*index] = false;
    }
    {
        let mut visitor = LintVisitor::with_rule_filter(ctx, rules, names, exit, &keep);
        profile!("patina.template.visit", visitor.visit_root(input.root));
    }
    let lowered = profile!(
        "patina.sfc.facade.lower",
        S2Template::lower(input.allocator, input.source)
    );
    let markup = lowered.markup();
    let markup = crate::markup::reborrow_markup(&markup);
    let mut document = MarkupDocument::from_s2(markup, TemplateSyntax::Vue);
    if let Some(analysis) = input.analysis {
        document = document.with_analysis(analysis);
    }
    profile!("patina.sfc.facade.visit", {
        let mut markup_ctx = MarkupContext::new(ctx, &document);
        document.visit_with(&batch::Rules(&selected), &mut markup_ctx);
    });
}

#[cfg(test)]
mod tests;
