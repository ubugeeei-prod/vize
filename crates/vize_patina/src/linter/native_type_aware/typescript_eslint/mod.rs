//! typescript-eslint type-aware checks on SFC script blocks.
mod facts;
mod gaps;
#[cfg(test)]
mod mapping_tests;
mod names;
mod options;
mod present;
mod tree;

use corsa::lint::{default_type_aware_registry, run_default_type_aware_rule_owned};
use vize_atelier_sfc::SfcDescriptor;
use vize_canon::virtual_ts::ProjectionMapping;
use vize_l0::{FxHashSet, String as CompactString};

use super::{CorsaTypeAwareSession, LintResult, Linter, push_warning};
use crate::diagnostic::LintDiagnostic;
use crate::linter::corsa_session::{ExpressionFact, ExpressionQuery};
use facts::queries;
use names::{RULES, corsa_name, static_rule};

pub(super) struct Script {
    content: CompactString,
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
                content: CompactString::new(&block.content),
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
                    .map(|listener| listener.as_str())
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
        .collect::<FxHashSet<_>>();
    listeners.insert("TSTypeAliasDeclaration");
    listeners.insert("VariableDeclarator");
    let listener_list = listeners.iter().copied().collect::<Vec<_>>();
    for script in &plan.scripts {
        let mut collected = tree::collect(&script.content, script.tsx, &listener_list);
        facts::prepare(&script.content, &mut collected);
        let script_queries = queries(&collected.nodes);
        let mut generated = Vec::new();
        let mut used = Vec::new();
        for (index, query) in script_queries.iter().enumerate() {
            let Some((start, end)) = generated_range(
                &plan.mapping,
                virtual_content,
                script,
                query.start,
                query.end,
            ) else {
                continue;
            };
            generated.push(ExpressionQuery {
                start,
                end,
                symbols: query.symbols,
                signatures: query.signatures,
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
        let mut seen = FxHashSet::default();
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
                let mut prepared = present::prepare(rule, node);
                if let Some(configured) = options::for_rule(&linter.type_aware_options, rule) {
                    prepared.fields.insert("__ruleOptions".into(), configured);
                }
                for diagnostic in
                    run_default_type_aware_rule_owned(rule, prepared).unwrap_or_default()
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

/// The generated range that copies script range `start..end` verbatim.
///
/// Rows can link the same authored bytes twice: a macro name also maps to the
/// prop key its declaration emits, so `defineModel` links to `"modelValue"` in
/// the props type ahead of the setup statement that copies it. A caret lookup
/// takes the first such row, and mapping a range's two ends one at a time can
/// start in the props type and end in setup. Only one row that holds the whole
/// range, and whose generated bytes equal the authored ones, answers for it.
fn generated_range(
    mapping: &ProjectionMapping,
    generated: &str,
    script: &Script,
    start: u32,
    end: u32,
) -> Option<(u32, u32)> {
    if end <= start {
        return None;
    }
    let authored = script.content.get(start as usize..end as usize)?;
    let authored_start = (script.offset as usize)
        .checked_add(start as usize)?
        .checked_sub(mapping.authored_base())?;
    let authored_end = authored_start.checked_add(authored.len())?;
    mapping.spans().iter().find_map(|span| {
        if authored_start < span.src_range.start || span.src_range.end < authored_end {
            return None;
        }
        let generated_start = span
            .gen_range
            .start
            .checked_add(authored_start - span.src_range.start)?;
        let generated_end = generated_start.checked_add(authored.len())?;
        if generated_end > span.gen_range.end
            || generated.get(generated_start..generated_end)? != authored
        {
            return None;
        }
        Some((
            u32::try_from(generated_start).ok()?,
            u32::try_from(generated_end).ok()?,
        ))
    })
}
