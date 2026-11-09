//! Rule-name sets that gate shared analysis work in the lint engine.
//!
//! A rule must appear in a set to receive the work that set unlocks:
//!
//! - [`SEMANTIC_TEMPLATE_RULES`]: rules whose `run_on_template` pass needs the
//!   template semantic analysis (`LintContext::has_analysis`). Omitting a rule
//!   here means its template pass returns before the rule body runs.
//! - [`SHARED_SFC_DESCRIPTOR_RULES`]: rules that consume SFC descriptor
//!   metadata, so the outer SFC path parses the descriptor once up front
//!   instead of letting each rule re-parse the file.

pub(super) const SEMANTIC_TEMPLATE_RULES: &[&str] = &[
    "vue/no-template-shadow",
    "vue/no-unused-vars",
    "vue/no-unused-setup-bindings",
    "vue/no-unused-components",
    "vue/require-component-registration",
    "vue/component-name-in-template-casing",
    "vue/no-undefined-refs",
    "vue/no-mutating-props",
    "vue/no-unused-properties",
    "vue/no-unused-refs",
    "vue/prop-name-casing",
    "a11y/no-refer-to-non-existent-id",
    "ecosystem/router-link-require-to",
    "ssr/no-browser-globals-in-ssr",
];

pub(super) const SHARED_SFC_DESCRIPTOR_RULES: &[&str] = &[
    "vue/html-quotes",
    "vue/match-component-file-name",
    "vue/multi-word-component-names",
    "vue/no-array-index-key",
    "vue/no-deprecated-functional-template",
    "ssr/no-browser-globals-in-ssr",
    "vue/no-mutating-props",
    "vue/no-reserved-component-names",
    "vue/no-root-v-if",
    "vue/no-unused-properties",
    "vue/no-unused-refs",
    "vue/prop-name-casing",
    "vue/max-template-complexity",
    "vue/no-unused-setup-bindings",
    "vue/sfc-element-order",
    "vue/require-scoped-style",
    "vue/single-style-block",
    "vue/valid-template-root",
    "vue/no-src-attribute",
    "vue/no-preprocessor-lang",
    "vue/no-template-lang",
    "vue/warn-custom-block",
    "vue/warn-custom-directive",
    "a11y/no-redundant-roles",
    "ecosystem/void-link-require-href",
    "ecosystem/void-link-valid-method",
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{SEMANTIC_TEMPLATE_RULES, SHARED_SFC_DESCRIPTOR_RULES};
    use crate::rule::RuleRegistry;

    #[test]
    fn engine_rule_name_sets_only_name_dispatchable_rules() {
        let mut available: BTreeSet<_> = RuleRegistry::with_all()
            .rule_names()
            .iter()
            .copied()
            .collect();
        available.extend(
            RuleRegistry::with_opt_in_rules()
                .rule_names()
                .iter()
                .copied(),
        );
        let missing: Vec<&str> = SEMANTIC_TEMPLATE_RULES
            .iter()
            .chain(SHARED_SFC_DESCRIPTOR_RULES)
            .copied()
            .filter(|name| !available.contains(name))
            .collect();
        assert_eq!(
            missing,
            Vec::<&str>::new(),
            "engine rule-name sets must only name rules a registry can instantiate"
        );
    }
}

impl crate::linter::config::Linter {
    pub(crate) fn has_unused_bindings_demand(&self) -> bool {
        self.registry.has_rule("vue/no-unused-setup-bindings")
            && self.is_rule_enabled("vue/no-unused-setup-bindings")
    }
}

use crate::linter::sfc_scripts::SfcScripts;
use crate::markup::L2Template;
use vize_atelier_sfc::croquis::SfcCroquisOptions;
use vize_croquis::Croquis;
use vize_l0::dialect::VueDialect;

/// Document-level inputs shared by the template-rule passes.
///
/// Bundles the optional SFC descriptor with the resolved [`VueDialect`] so the
/// rule context can gate dialect-specific rules (e.g. petite-vue keyless
/// `v-for`) without growing the already-wide pass signatures.
#[derive(Clone, Copy)]
pub(crate) struct TemplateRuleEnv<'a> {
    pub sfc_descriptor: Option<&'a vize_atelier_sfc::SfcDescriptor<'a>>,
    pub sfc_scripts: Option<&'a dyn crate::linter::sfc_scripts::ScriptParses>,
    pub art_script_analysis: Option<&'a Croquis>,
    /// Completed Davinci petite-vue meaning for standalone HTML, when admitted.
    pub petite_document: Option<&'a vize_l2::dialect::petite::PetiteDocument<'a, 'a>>,
    /// Exact-node pragmas are applied after all complete-SFC producers join.
    pub defer_exact_suppressions: bool,
    pub dialect: VueDialect,
    /// Markup rules `lint_sfc` runs on the L2 facade instead of the visitor.
    /// Empty on the raw-template and standalone-HTML lanes.
    pub facade_rules: &'static [&'static str],
}

pub(crate) fn analyze_descriptor_for_lint(
    scripts: &SfcScripts<'_>,
    template: Option<&L2Template<'_>>,
    resolve_filename: Option<&str>,
    unused: bool,
    derived: bool,
) -> Croquis {
    analyze_descriptor_for_lint_with_template(scripts, template, resolve_filename, unused, derived)
}

pub(crate) fn analyze_descriptor_for_lint_with_template(
    scripts: &SfcScripts<'_>,
    template: Option<&L2Template<'_>>,
    resolve_filename: Option<&str>,
    unused: bool,
    derived: bool,
) -> Croquis {
    let options = SfcCroquisOptions::lint_demand();
    let options = if unused {
        options.with_unused_bindings()
    } else {
        options
    };
    let options = if derived {
        options.with_derived_template()
    } else {
        options
    };
    let template = template.map(crate::markup::L2Template::lint_template);
    match resolve_filename {
        Some(filename) => {
            vize_atelier_sfc::croquis::analyze_sfc_descriptor_with_lint_template_resolved(
                scripts, template, options, filename,
            )
        }
        None => vize_atelier_sfc::croquis::analyze_sfc_descriptor_with_lint_template(
            scripts, template, options,
        ),
    }
}

/// Request positive authored reads within the existing single descriptor draw.
pub(super) fn analyze_descriptor_for_component_reads(
    scripts: &SfcScripts<'_>,
    template: Option<&L2Template<'_>>,
    resolve_filename: Option<&str>,
    unused: bool,
    derived: bool,
) -> Croquis {
    if scripts
        .descriptor()
        .script_setup
        .as_ref()
        .is_none_or(|script| {
            script.src.is_some()
                || script
                    .lang
                    .as_deref()
                    .is_some_and(|lang| !matches!(lang, "js" | "ts" | "jsx" | "tsx"))
        })
    {
        return analyze_descriptor_for_lint_with_template(
            scripts,
            template,
            resolve_filename,
            unused,
            derived,
        );
    }
    let options = SfcCroquisOptions::lint_demand();
    let options = if unused {
        options.with_unused_bindings()
    } else {
        options
    };
    let options = if derived {
        options.with_derived_template()
    } else {
        options
    };
    let template = template.map(crate::markup::L2Template::lint_template);
    let (mut analysis, packet) = match resolve_filename {
        Some(filename) => {
            vize_atelier_sfc::croquis::analyze_sfc_descriptor_with_lint_template_and_occurrences_resolved(
                scripts, template, options, filename,
            )
        }
        None => {
            vize_atelier_sfc::croquis::analyze_sfc_descriptor_with_lint_template_and_occurrences(
                scripts, template, options,
            )
        }
    };
    let component_reads = analysis.croquis.authored_component_reads_are_qualified();
    // Preserve the complete editor refusal separately from independently
    // source-witnessed positive reads admitted by the original SFC wrapper.
    analysis.croquis.retain_authored_binding_reads(packet);
    analysis
        .croquis
        .qualify_authored_component_reads(component_reads);
    analysis.croquis
}
