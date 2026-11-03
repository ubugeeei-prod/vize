//! SFC markup rules whose diagnostics match the Relief visitor (P4-7b).
//!
//! The admitted rules share one L1 parse and one traversal per declared
//! level: rules that declare L1 walk the L1 document, and the L2 lowering
//! is built only when an enabled rule declares L2. All other template rules
//! stay on the Relief visitor until their parity is proven.

use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::linter::config::Linter;
use crate::markup::{L1Markup, L2Template, MarkupContext, MarkupDocument, MarkupLevel};
use crate::visitor::LintVisitor;
use vize_croquis::Croquis;
use vize_l0::{Allocator, profile};
use vize_relief::RootNode;

mod batch;

// This rule's whole-document check replaces a legacy template-level callback.
// Keep that callback at its registry position before the element visitor.
const TEMPLATE_DOCUMENT_RULE: &str = "vue/permitted-contents";

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
    "vapor/no-vue-lifecycle-events",
    "vue/no-static-inline-styles",
    "a11y/no-access-key",
    "a11y/no-autofocus",
    "vue/require-v-for-key",
    "vapor/prefer-static-class",
    "a11y/no-redundant-roles",
    "a11y/interactive-supports-focus",
    "vue/no-bare-strings-in-template",
    "vue/permitted-contents",
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
    let all_rules = linter.registry.rules();
    let rules = all_rules.get(..input.rule_count).unwrap_or(all_rules);
    let all_names = linter.rule_names();
    let names = all_names.get(..input.rule_count).unwrap_or(all_names);
    let exit = linter.registry.has_exit_element_rules();
    let selected: Vec<_> = rules
        .iter()
        .zip(names.iter().copied())
        .enumerate()
        .filter_map(|(index, (rule, name))| {
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
        if let Some(slot) = keep.get_mut(*index) {
            *slot = false;
        }
    }
    let Some(l1) = profile!(
        "patina.sfc.facade.parse",
        L1Markup::parse(input.allocator, input.source)
    ) else {
        // A source past `u32` offsets parses to one unexpected hole, so no
        // markup rule has a node to see.
        let mut visitor = LintVisitor::with_rule_filter(ctx, rules, names, exit, &keep);
        profile!("patina.template.visit", visitor.visit_root(input.root));
        return;
    };
    let (surface_rules, lowered_rules): (Vec<_>, Vec<_>) = selected
        .iter()
        .copied()
        .partition(|(_, _, rule)| rule.level() == MarkupLevel::L1);
    let lowered = (!lowered_rules.is_empty()).then(|| {
        profile!(
            "patina.sfc.facade.lower",
            L2Template::from_l1(input.allocator, l1)
        )
    });
    let lowered_markup = lowered.as_ref().map(L2Template::markup);
    let with_analysis = |document: MarkupDocument<'a>| match input.analysis {
        Some(analysis) => document.with_analysis(analysis),
        None => document,
    };
    let surface = with_analysis(MarkupDocument::from_l1(l1, TemplateSyntax::Vue));
    let lowered_document = lowered_markup.as_ref().map(|markup| {
        let markup = crate::markup::reborrow_markup(markup);
        with_analysis(MarkupDocument::from_l2(markup, TemplateSyntax::Vue))
    });
    let is_document_rule = |(_, name, _): &&(usize, &str, _)| *name == TEMPLATE_DOCUMENT_RULE;
    let document_rule = surface_rules
        .iter()
        .find(is_document_rule)
        .map(|selected| (selected, surface))
        .or_else(|| {
            lowered_rules
                .iter()
                .find(is_document_rule)
                .zip(lowered_document)
        });
    {
        let mut visitor = LintVisitor::with_rule_filter(ctx, rules, names, exit, &keep);
        profile!("patina.template.visit", {
            visitor.visit_root_with_template_hook(input.root, &mut |index, ctx| {
                let Some(((selected_index, _, rule), document)) = document_rule else {
                    return false;
                };
                if index != *selected_index {
                    return false;
                }
                let mut markup_ctx = MarkupContext::new(ctx, &document);
                rule.enter_document(&mut markup_ctx, &document);
                true
            });
        });
    }
    profile!("patina.sfc.facade.visit", {
        let mut markup_ctx = MarkupContext::new(ctx, &surface);
        surface.visit_with(&batch::Rules(&surface_rules), &mut markup_ctx);
        if let Some(document) = lowered_document {
            let mut markup_ctx = MarkupContext::new(ctx, &document);
            document.visit_with(&batch::Rules(&lowered_rules), &mut markup_ctx);
        }
    });
}

#[cfg(test)]
mod tests;
