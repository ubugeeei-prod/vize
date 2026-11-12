//! typescript-eslint type-aware checks on SFC script blocks.
mod facts;
mod gaps;
mod names;
mod present;
mod tree;

use std::collections::HashSet;

use corsa::lint::{default_type_aware_registry, run_default_type_aware_rule_owned};
use vize_atelier_sfc::SfcDescriptor;
use vize_canon::virtual_ts::ProjectionMapping;
use vize_l0::String as CompactString;

use super::{CorsaTypeAwareSession, LintResult, Linter, RULE_NO_FLOATING_PROMISES, push_warning};
use crate::diagnostic::LintDiagnostic;
use crate::linter::corsa_session::{ExpressionFact, ExpressionQuery};
use corsa::lint::LintNode;
use facts::{PromiseRules, queries};
use names::{RULES, corsa_name, static_rule};

pub(super) struct Script {
    content: String,
    offset: u32,
    tsx: bool,
}

pub(super) struct Plan {
    pub scripts: Vec<Script>,
    pub mapping: ProjectionMapping,
}

impl Plan {
    pub(super) fn from_descriptor(
        descriptor: &SfcDescriptor<'_>,
        mapping: ProjectionMapping,
    ) -> Self {
        let scripts = descriptor
            .script
            .iter()
            .chain(descriptor.script_setup.iter())
            .filter(|block| !block.content.is_empty())
            .map(|block| Script {
                content: block.content.to_string(),
                offset: block.loc.start as u32,
                tsx: matches!(block.lang.as_deref().map(str::trim), Some("tsx" | "jsx")),
            })
            .collect();
        Self { scripts, mapping }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.scripts.is_empty()
    }
}

pub(super) fn is_active(linter: &Linter) -> bool {
    RULES
        .iter()
        .any(|(name, _)| linter.registry.has_rule(name) && linter.is_rule_enabled(name))
        || floating_active(linter)
}

/// `type/no-floating-promises` is a native vize rule; on SFC scripts it runs
/// typescript-eslint's decision flow over checker facts here.
pub(super) fn floating_active(linter: &Linter) -> bool {
    linter.registry.has_rule(RULE_NO_FLOATING_PROMISES)
        && linter.is_rule_enabled(RULE_NO_FLOATING_PROMISES)
}

pub(super) fn evaluate(
    session: &CorsaTypeAwareSession,
    plan: &Plan,
    virtual_content: &str,
    linter: &Linter,
    result: &mut LintResult,
) -> Result<(), CompactString> {
    if plan.scripts.is_empty() {
        return Ok(());
    }
    let enabled = RULES
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| linter.registry.has_rule(name) && linter.is_rule_enabled(name))
        .collect::<Vec<_>>();
    let corsa_rules = enabled
        .iter()
        .copied()
        .filter_map(corsa_name)
        .collect::<Vec<_>>();
    let metas = default_type_aware_registry().metas();
    let mut listeners = corsa_rules
        .iter()
        .filter_map(|name| {
            metas.iter().find(|meta| meta.name == *name).map(|meta| {
                meta.listeners
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
        .collect::<HashSet<_>>();
    listeners.insert("TSTypeAliasDeclaration");
    listeners.insert("VariableDeclarator");
    let floating = floating_active(linter);
    if floating {
        listeners.insert("ExpressionStatement");
    }
    let promise_rules = PromiseRules::from_rules(&corsa_rules, floating);
    if promise_rules.any() {
        // Promise executors are found from their `new` expression, and object
        // literal methods from their object's contextual type.
        listeners.insert("NewExpression");
        listeners.insert("ObjectExpression");
    }
    let listener_list = listeners.iter().copied().collect::<Vec<_>>();
    for script in &plan.scripts {
        let mut collected = tree::collect(&script.content, script.tsx, &listener_list);
        facts::prepare(&script.content, &mut collected);
        let script_queries = queries(&collected.nodes, promise_rules);
        let mut generated = Vec::new();
        let mut used = Vec::new();
        for (index, query) in script_queries.iter().enumerate() {
            let Some((start, end)) =
                generated_range(&plan.mapping, script.offset, query.start, query.end)
            else {
                continue;
            };
            generated.push(ExpressionQuery {
                start,
                end,
                symbols: query.symbols,
                signatures: query.signatures,
                promise: query.promise,
                contextual: query.contextual,
                void_arguments: query.void_arguments,
                members: query.members.clone(),
                heritage: query.heritage.clone(),
            });
            used.push(index);
        }
        let probed = session.expression_facts(virtual_content, &generated)?;
        let mut aligned = vec![ExpressionFact::default(); script_queries.len()];
        for (index, fact) in used.into_iter().zip(probed) {
            if let Some(slot) = aligned.get_mut(index) {
                *slot = fact;
            }
        }
        facts::apply(&mut collected.nodes, &aligned, &script_queries);
        facts::enrich(&script.content, &mut collected.nodes, &collected.program);
        let mut seen = HashSet::new();
        if floating {
            for node in collected
                .nodes
                .iter()
                .filter(|node| node.kind == "ExpressionStatement")
            {
                for diagnostic in floating_reports(node) {
                    let start = script.offset.saturating_add(diagnostic.range.start);
                    let end = script.offset.saturating_add(diagnostic.range.end);
                    if seen.insert((
                        RULE_NO_FLOATING_PROMISES,
                        start,
                        end,
                        diagnostic.message.clone(),
                    )) {
                        push_warning(
                            result,
                            LintDiagnostic::warn(
                                RULE_NO_FLOATING_PROMISES,
                                diagnostic.message,
                                start,
                                end,
                            )
                            .with_help(
                                "Add `await`, return the Promise, or prefix it with `void` when the fire-and-forget behavior is intentional.",
                            ),
                        );
                    }
                }
            }
        }
        for node in &collected.nodes {
            for rule in &corsa_rules {
                let listens = metas.iter().any(|meta| {
                    meta.name == *rule && meta.listeners.iter().any(|kind| kind == &node.kind)
                });
                if !listens {
                    continue;
                }
                let Some(vize_name) = static_rule(rule) else {
                    continue;
                };
                for diagnostic in
                    run_default_type_aware_rule_owned(rule, present::prepare(rule, node))
                        .unwrap_or_default()
                {
                    let (raw_start, raw_end) =
                        present::range(rule, node, diagnostic.range.start, diagnostic.range.end);
                    let start = script.offset.saturating_add(raw_start);
                    let end = script.offset.saturating_add(raw_end);
                    if seen.insert((vize_name, start, end, diagnostic.message.clone())) {
                        push_warning(
                            result,
                            LintDiagnostic::warn(vize_name, diagnostic.message, start, end),
                        );
                    }
                }
            }
        }
        for gap in gaps::reports(&collected) {
            let start = script.offset.saturating_add(gap.start);
            let end = script.offset.saturating_add(gap.end);
            if seen.insert((gap.rule, start, end, gap.message.to_owned())) {
                push_warning(
                    result,
                    LintDiagnostic::warn(gap.rule, gap.message, start, end),
                );
            }
        }
    }
    Ok(())
}

/// typescript-eslint's no-floating-promises on one statement. Upstream
/// reports the statement; vize marks its expression.
fn floating_reports(node: &LintNode) -> Vec<corsa::lint::LintDiagnostic> {
    let Some(expression) = node.child("expression") else {
        return Vec::new();
    };
    let mut diagnostics =
        run_default_type_aware_rule_owned("no-floating-promises", node.clone()).unwrap_or_default();
    for diagnostic in &mut diagnostics {
        diagnostic.range = expression.range;
    }
    diagnostics
}

fn generated_range(
    mapping: &ProjectionMapping,
    base: u32,
    start: u32,
    end: u32,
) -> Option<(u32, u32)> {
    if end <= start {
        return None;
    }
    let file_start = base.checked_add(start)? as usize;
    let file_last = base.checked_add(end - 1)? as usize;
    let generated_start = u32::try_from(mapping.to_generated(file_start)?).ok()?;
    let generated_last = u32::try_from(mapping.to_generated(file_last)?).ok()?;
    Some((generated_start, generated_last.checked_add(1)?))
}
