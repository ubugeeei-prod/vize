//! `vue/no-unused-setup-bindings` (P4-3c), opt-in, tier Sound.
//!
//! Report valid script-setup declarations with no resolved script, template
//! or style `v-bind()` read. Underscore prefixes express intentional non-use.
//! Croquis stores the authoritative relation. A binding used only as a `v-for`
//! source is still a read; the template walk below counts those when the
//! stored relation missed the source expression. Unknown/invalid or external
//! blocks are outside the domain.

use vize_croquis::drawer::{extract_identifiers_oxc, parse_v_for_scope_expression};
use vize_croquis::facts::{Demand, FactConsumer, FactGroup, UnusedBindings};
use vize_l0::{CompactString, FxHashSet};
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};

static META: RuleMeta = RuleMeta {
    name: "vue/no-unused-setup-bindings",
    description: "Disallow unread script setup bindings",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Opt-in rule over the UnusedBindings group.
#[derive(Default)]
pub struct NoUnusedSetupBindings;

impl FactConsumer for NoUnusedSetupBindings {
    const NAME: &'static str = "vue/no-unused-setup-bindings";
    const DEMAND: Demand = Demand::NONE.with(UnusedBindings::ID);
}

impl NoUnusedSetupBindings {
    pub(crate) fn report(ctx: &mut LintContext<'_>, root: Option<&RootNode<'_>>) {
        let Some(view) = ctx.facts::<Self>() else {
            return;
        };
        let Ok(table) = view.get::<UnusedBindings>() else {
            return;
        };
        let v_for_sources = root.map(v_for_source_reads).unwrap_or_default();
        let diagnostics = diagnostics(table, &v_for_sources);
        for diagnostic in diagnostics {
            ctx.report_in_script(diagnostic);
        }
    }
}

impl Rule for NoUnusedSetupBindings {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        if ctx.sfc_descriptor().is_some() && !has_art_variants(ctx) {
            Self::report(ctx, Some(root));
        }
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        if !ctx.is_rule_enabled(META.name) {
            return;
        }
        if has_art_variants(ctx) {
            // The Art pass reports once after every original variant has read
            // the shared physical script's candidate relation.
            return;
        }
        let Some(descriptor) = ctx.sfc_descriptor() else {
            return;
        };
        if descriptor.template.is_some() {
            return;
        }
        // A script-only SFC never reaches the template analysis path. Draw
        // exactly once for that artifact using the shared descriptor.
        let analysis = vize_atelier_sfc::croquis::analyze_sfc_descriptor(
            descriptor,
            None,
            vize_atelier_sfc::croquis::SfcCroquisOptions::lint_demand().with_unused_bindings(),
        );
        let mut facts = vize_croquis::facts::CroquisFacts::new(&analysis);
        let Ok(table) = facts.prepare::<Self>().get::<UnusedBindings>() else {
            return;
        };
        let diagnostics = diagnostics(table, &FxHashSet::default());
        for diagnostic in diagnostics {
            ctx.report_in_script(diagnostic);
        }
    }
}

fn diagnostics(
    table: &vize_croquis::facts::FactTable<UnusedBindings>,
    v_for_sources: &FxHashSet<CompactString>,
) -> Vec<LintDiagnostic> {
    table
        .iter()
        .filter(|(name, _)| {
            let name: &str = name.as_ref();
            !name.starts_with('_') && !v_for_sources.contains(name)
        })
        .map(|(name, fact)| {
            LintDiagnostic::warn(
                META.name,
                vize_l0::cstr!("Setup binding '{name}' is never read"),
                fact.span.0,
                fact.span.1,
            )
            .with_help(
                "Remove the binding or prefix its name with underscore if intentionally unused",
            )
        })
        .collect()
}

/// Identifiers read by `v-for` source expressions (`item in DAYS`, `(row, at) in rows`).
pub(crate) fn v_for_source_reads(root: &RootNode<'_>) -> FxHashSet<CompactString> {
    let mut reads = FxHashSet::default();
    walk_v_for_sources(&root.children, root.source, &mut reads);
    reads
}

fn has_art_variants(ctx: &LintContext<'_>) -> bool {
    ctx.filename.ends_with(".art.vue")
        && ctx.sfc_descriptor().is_some_and(|descriptor| {
            descriptor
                .custom_blocks
                .iter()
                .any(|block| block.block_type.as_ref() == "art")
        })
}

fn walk_v_for_sources(
    children: &[TemplateChildNode<'_>],
    source: &str,
    reads: &mut FxHashSet<CompactString>,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                for prop in &element.props {
                    if let PropNode::Directive(directive) = prop {
                        push_v_for_directive(directive, source, reads);
                    }
                }
                walk_v_for_sources(&element.children, source, reads);
            }
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    walk_v_for_sources(&branch.children, source, reads);
                }
            }
            TemplateChildNode::IfBranch(node) => {
                walk_v_for_sources(&node.children, source, reads);
            }
            TemplateChildNode::For(node) => {
                push_expression_reads(&node.source, source, reads);
                walk_v_for_sources(&node.children, source, reads);
            }
            _ => {}
        }
    }
}

fn push_v_for_directive(
    directive: &vize_relief::DirectiveNode<'_>,
    source: &str,
    reads: &mut FxHashSet<CompactString>,
) {
    if directive.name != "for" {
        return;
    }
    if let Some(parsed) = &directive.for_parse_result {
        push_expression_reads(&parsed.source, source, reads);
        return;
    }
    let Some(exp) = directive.exp.as_ref() else {
        return;
    };
    let text = expression_text(exp, source);
    let Some(aliases) = parse_v_for_scope_expression(text) else {
        return;
    };
    for ident in extract_identifiers_oxc(aliases.source.as_str()) {
        reads.insert(ident);
    }
}

fn push_expression_reads(
    exp: &ExpressionNode<'_>,
    source: &str,
    reads: &mut FxHashSet<CompactString>,
) {
    for ident in extract_identifiers_oxc(expression_text(exp, source)) {
        reads.insert(ident);
    }
}

fn expression_text<'a>(exp: &'a ExpressionNode<'a>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(simple) => simple.content,
        ExpressionNode::Compound(compound) => compound.loc.span.slice(source),
    }
}

#[cfg(test)]
mod tests;
