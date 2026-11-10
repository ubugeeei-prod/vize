//! SFC markup rules whose diagnostics match the Relief visitor (P4-7b).
//!
//! The admitted rules share one L2 lowering and one traversal. All other
//! template rules stay on the Relief visitor until their parity is proven.

use crate::context::{LintContext, TemplateCommentDirectives};
use crate::ir::TemplateSyntax;
use crate::linter::config::Linter;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRules};
use crate::visitor::LintVisitor;
use vize_croquis::Croquis;
use vize_l0::profile;
use vize_relief::RootNode;

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
    "vue/prefer-props-shorthand",
    "vue/no-useless-v-bind",
    "vue/no-multiple-objects-in-class",
    "vue/slot-name-casing",
    "vue/use-unique-element-ids",
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
    "a11y/alt-text",
    "a11y/anchor-has-content",
    "a11y/aria-props",
    "a11y/aria-role",
    "a11y/aria-unsupported-elements",
    "a11y/click-events-have-key-events",
    "a11y/label-has-for",
    "a11y/no-refer-to-non-existent-id",
    "a11y/no-static-element-interactions",
    "a11y/role-has-required-aria-props",
    "vue/valid-v-cloak",
    "vue/valid-v-once",
    "vue/valid-v-text",
    "vue/valid-v-html",
    "vue/valid-v-show",
    "vue/no-v-text",
];

pub(in crate::linter::engine) struct Dispatch<'a> {
    pub root: &'a RootNode<'a>,
    pub l2: &'a crate::markup::L2Template<'a>,
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
    let directives = TemplateCommentDirectives::new(input.l2.surface());
    let selected = linter.markup_rules(input.rule_count, |name, _| input.rules.contains(&name));

    if selected.is_empty() {
        let mut visitor =
            LintVisitor::new(ctx, rules, names, exit).with_template_directives(&directives);
        profile!("patina.template.visit", visitor.visit_root(input.root));
        return;
    }

    let mut keep = vec![true; input.rule_count];
    for rule in &selected {
        if let Some(slot) = keep.get_mut(rule.rank()) {
            *slot = false;
        }
    }
    let markup = input.l2.markup();
    let markup = crate::markup::reborrow_markup(&markup);
    let mut document = MarkupDocument::from_l2(markup, TemplateSyntax::Vue);
    if let Some(analysis) = input.analysis {
        document = document.with_analysis(analysis);
    }
    {
        let mut visitor = LintVisitor::with_rule_filter(ctx, rules, names, exit, &keep)
            .with_template_directives(&directives);
        profile!("patina.template.visit", visitor.visit_root(input.root));
    }
    profile!("patina.sfc.facade.visit", {
        let mut markup_ctx = MarkupContext::new(ctx, &document);
        document.visit_with(&MarkupRules::new(&selected), &mut markup_ctx);
    });
}

#[cfg(test)]
mod tests;
