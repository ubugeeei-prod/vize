//! The P4-7b switch oracle. Every markup-capable rule runs three ways over one
//! template:
//!
//! - **legacy** — its `Rule` hooks over the lint parse, the way the template
//!   lane ran it before the switch;
//! - **relief** — its `MarkupRule` body over the Relief-backed facade, fused
//!   with the other markup rules exactly as the production markup lane runs
//!   it after the switch;
//! - **s2** — the same bodies over the S2-backed facade, the backend the
//!   markup lane moves to once the Relief parse retires.
//!
//! The relief lane must reproduce the legacy diagnostic set — rule, severity,
//! message, span, help, labels and fix — on every template: that is the
//! switch. The s2 lane must too, except on a template the lint parse
//! restructured (HTML tree construction; see [`super::nesting`]), where the
//! two backends present different documents by design; such templates are
//! counted, never compared.
//!
//! This is the local form of "TS-9 lint snapshots unchanged": the snapshot
//! corpora only see what these lanes produce on real templates.

use super::nesting;
use crate::context::LintContext;
use crate::diagnostic::LintDiagnostic;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRuleSet, S2Template};
use crate::rule::{Rule, RuleRegistry};
use crate::visitor::LintVisitor;
use vize_s0::{Allocator, String, append};

/// The facade backend a markup body disagreed with its legacy hooks on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchLane {
    /// The Relief-backed facade: the production markup lane.
    Relief,
    /// The S2-backed facade.
    S2,
}

/// One rule whose markup body disagreed with its legacy hooks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneDivergence {
    /// The rule.
    pub rule: &'static str,
    /// The backend it disagreed on.
    pub lane: SwitchLane,
    /// Its legacy diagnostics, rendered and sorted.
    pub legacy: std::vec::Vec<String>,
    /// Its markup-body diagnostics on `lane`, rendered and sorted.
    pub markup: std::vec::Vec<String>,
}

/// What the three lanes of every rule produced over one template.
#[derive(Debug, Default)]
pub struct SwitchReport {
    /// Legacy diagnostics the relief lane reproduced exactly.
    pub agreed: usize,
    /// Legacy diagnostics the s2 lane reproduced exactly (0 when restructured).
    pub s2_agreed: usize,
    /// The lint parse restructured the authored tree: the s2 lane is not
    /// compared.
    pub restructured: bool,
    /// Every disagreement.
    pub divergences: std::vec::Vec<LaneDivergence>,
}

/// Every markup-capable rule a registry can instantiate, by name, once.
pub fn markup_registry() -> RuleRegistry {
    let mut registry = RuleRegistry::new();
    let mut seen: std::vec::Vec<&'static str> = std::vec::Vec::new();
    for source in [RuleRegistry::with_all(), RuleRegistry::with_opt_in_rules()] {
        for rule in owned_rules(source) {
            let name = rule.meta().name;
            if rule.as_markup_rule().is_some()
                && rule.markup_on_templates()
                && !seen.contains(&name)
            {
                seen.push(name);
                registry.register(rule);
            }
        }
    }
    registry
}

fn owned_rules(registry: RuleRegistry) -> std::vec::Vec<Box<dyn Rule>> {
    registry.rules
}

fn render(diagnostics: &[LintDiagnostic], rule: &str) -> std::vec::Vec<String> {
    let mut lines: std::vec::Vec<String> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == rule)
        .map(|diagnostic| {
            let mut line = String::default();
            append!(line, "{diagnostic:?}");
            line
        })
        .collect();
    lines.sort();
    lines
}

/// The three lanes of every rule in `registry` over a bare template.
pub fn rule_lanes(registry: &RuleRegistry, source: &str) -> SwitchReport {
    rule_lanes_with(registry, source, None)
}

/// The three lanes over a whole SFC's template, with its descriptor attached
/// as the SFC lint path attaches it (container-aware rules read the style
/// blocks). An SFC without a template reports nothing.
pub fn sfc_rule_lanes(registry: &RuleRegistry, sfc: &str) -> SwitchReport {
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(sfc, Default::default()) else {
        return SwitchReport::default();
    };
    let Some(template) = descriptor.template.as_ref() else {
        return SwitchReport::default();
    };
    rule_lanes_with(registry, &template.content, Some(&descriptor))
}

fn context<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    descriptor: Option<&'a vize_atelier_sfc::SfcDescriptor<'a>>,
) -> LintContext<'a> {
    let mut ctx = LintContext::new(allocator, source, "switch.vue");
    if let Some(descriptor) = descriptor {
        ctx.set_sfc_template_descriptor(descriptor);
    }
    ctx
}

fn markup_lane<'a>(
    ctx: &mut LintContext<'a>,
    document: &MarkupDocument<'a>,
    rules: &MarkupRuleSet<'_>,
) {
    let mut markup_ctx = MarkupContext::new(ctx, document);
    document.visit_rules(rules, &mut markup_ctx);
}

fn rule_lanes_with<'a>(
    registry: &RuleRegistry,
    source: &'a str,
    descriptor: Option<&'a vize_atelier_sfc::SfcDescriptor<'a>>,
) -> SwitchReport {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let (root, _errors) = vize_armature::Parser::new(&allocator, source).parse();
    let root = allocator.alloc(root);
    let mut legacy = context(&allocator, source, descriptor);
    LintVisitor::new(
        &mut legacy,
        registry.rules(),
        registry.rule_names(),
        registry.has_exit_element_rules(),
    )
    .visit_root(root);
    let legacy = legacy.into_diagnostics();

    let rules = MarkupRuleSet::new(
        registry
            .rules()
            .iter()
            .filter_map(|rule| rule.as_markup_rule()),
    );
    let mut relief = context(&allocator, source, descriptor);
    markup_lane(
        &mut relief,
        &MarkupDocument::new(root, TemplateSyntax::Vue),
        &rules,
    );
    let relief = relief.into_diagnostics();

    let lowered = S2Template::lower(&allocator, source);
    let restructured = nesting::is_restructured(root, lowered.surface());
    let markup = lowered.markup();
    let mut s2 = context(&allocator, source, descriptor);
    if !restructured {
        markup_lane(
            &mut s2,
            &MarkupDocument::from_s2(&markup, TemplateSyntax::Vue),
            &rules,
        );
    }
    let s2 = s2.into_diagnostics();

    let mut report = SwitchReport {
        restructured,
        ..SwitchReport::default()
    };
    for &rule in registry.rule_names() {
        let expected = render(&legacy, rule);
        let mut lanes = vec![(SwitchLane::Relief, render(&relief, rule))];
        if !restructured {
            lanes.push((SwitchLane::S2, render(&s2, rule)));
        }
        for (lane, markup) in lanes {
            if markup != expected {
                report.divergences.push(LaneDivergence {
                    rule,
                    lane,
                    legacy: expected.clone(),
                    markup,
                });
            } else if lane == SwitchLane::Relief {
                report.agreed += expected.len();
            } else {
                report.s2_agreed += expected.len();
            }
        }
    }
    report
}
