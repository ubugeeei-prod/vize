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
use vize_l0::{SmallVec, profile};
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
    "vue/no-deprecated-v-bind-sync",
    "vue/no-deprecated-v-on-native-modifier",
    "vue/no-deprecated-v-on-number-modifiers",
    "vue/no-deprecated-slot-attribute",
    "vue/no-deprecated-slot-scope-attribute",
    "vue/no-deprecated-scope-attribute",
    "vue/no-deprecated-inline-template",
    "vue/no-deprecated-router-link-tag-prop",
    "vue/no-deprecated-html-element-is",
    "vue/require-component-is",
    "vue/no-deprecated-v-is",
    "vue/v-on-style",
    "vue/v-slot-style",
    "vue/no-custom-modifiers-on-v-model",
    "vue/no-v-text-v-html-on-component",
    "vue/attribute-hyphenation",
    "vue/valid-attribute-name",
    "vue/valid-v-pre",
    "ecosystem/router-link-require-to",
    "ecosystem/void-link-require-href",
    "ecosystem/void-link-valid-method",
    "ecosystem/vue-router-prefer-named-link",
    "ecosystem/nuxt-prefer-nuxt-link",
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
    "a11y/role-img-has-name",
    "vue/a11y-img-alt",
    "vue/no-unsafe-url",
    "vapor/no-inline-template",
    "vue/no-restricted-static-attribute",
];

pub(in crate::linter::engine) struct Dispatch<'a> {
    pub root: &'a RootNode<'a>,
    pub l2: &'a crate::markup::LintInput<'a>,
    /// Where the facade keeps the L2 header it builds. The caller owns the
    /// slot, so the header lives as long as the lint context and the document
    /// can borrow it for `'a`. It stays empty when no facade rule runs.
    pub header: &'a mut Option<crate::markup::L2Markup<'a>>,
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
    let selected = linter.named_markup_rules(input.rule_count, input.rules);

    if selected.is_empty() {
        let mut visitor =
            LintVisitor::new(ctx, rules, names, exit).with_template_directives(&directives);
        profile!("patina.template.visit", visitor.visit_root(input.root));
        return;
    }

    let cached_legacy =
        (linter.enabled_rules.is_none() && linter.disabled_rules.is_empty()).then(|| {
            linter
                .registry
                .legacy_template_plan_named(input.rule_count, input.rules)
        });
    let mut legacy = SmallVec::<[usize; 128]>::new();
    if cached_legacy.is_none() {
        let mut selected_index = 0;
        for rank in 0..input.rule_count {
            while selected
                .get(selected_index)
                .is_some_and(|rule| rule.rank() < rank)
            {
                selected_index += 1;
            }
            if selected
                .get(selected_index)
                .is_some_and(|rule| rule.rank() == rank)
            {
                selected_index += 1;
            } else {
                legacy.push(rank);
            }
        }
    }
    let markup = &*input.header.insert(input.l2.markup());
    let mut document = MarkupDocument::from_l2(markup, TemplateSyntax::Vue);
    if let Some(analysis) = input.analysis {
        document = document.with_analysis(analysis);
    }
    {
        let mut visitor = if let Some(plan) = cached_legacy.as_deref() {
            LintVisitor::with_rule_plan(ctx, rules, names, exit, &plan.indices, &plan.hooks)
        } else {
            LintVisitor::with_rule_indices(ctx, rules, names, exit, &legacy)
        }
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
