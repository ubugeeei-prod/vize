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

use super::{CorsaTypeAwareSession, LintResult, Linter, push_warning};
use crate::diagnostic::LintDiagnostic;
use crate::linter::corsa_session::{
    ExpressionFact, ExpressionQuery, PredicateRequest as ShapeCall,
};
use facts::queries;
use names::{RULES, corsa_name, static_rule};

/// Rules that read the `__typeShape` (constrained type flags) fact.
const SHAPE_RULES: &[&str] = &[
    "no-unnecessary-condition",
    "no-unnecessary-template-expression",
    "no-unnecessary-type-conversion",
    "no-unnecessary-boolean-literal-compare",
];

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
    let listener_list = listeners.iter().copied().collect::<Vec<_>>();
    let shapes = corsa_rules.iter().any(|rule| SHAPE_RULES.contains(rule));
    for script in &plan.scripts {
        let mut collected = tree::collect(&script.content, script.tsx, &listener_list);
        facts::prepare(&script.content, &mut collected);
        let script_queries = queries(&collected.nodes, shapes);
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
                shape: query.shape.clone().map(|mut request| {
                    request.keys_from = request.keys_from.and_then(|(start, end)| {
                        generated_range(&plan.mapping, script.offset, start, end)
                    });
                    let map = |(start, end): (u32, u32)| {
                        generated_range(&plan.mapping, script.offset, start, end)
                    };
                    request.predicate = request.predicate.and_then(|call| {
                        Some(ShapeCall {
                            callee: map(call.callee)?,
                            arguments: call
                                .arguments
                                .into_iter()
                                .map(map)
                                .collect::<Option<Vec<_>>>()?,
                        })
                    });
                    request
                }),
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
        if session.no_unchecked_indexed_access() {
            for node in &mut collected.nodes {
                node.fields
                    .insert("__noUncheckedIndexedAccess".into(), serde_json::json!(true));
            }
        }
        let mut seen = HashSet::new();
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
