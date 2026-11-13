//! typescript-eslint type-aware checks on SFC script blocks.
mod facts;
mod gaps;
#[cfg(test)]
mod mapping_tests;
mod names;
mod options;
mod present;
mod relate;
mod tree;

use corsa::lint::{default_type_aware_registry, run_default_type_aware_rule_owned};
use vize_atelier_sfc::SfcDescriptor;
use vize_canon::virtual_ts::{ProjectionMapping, VizeMapping};
use vize_l0::{FxHashSet, String as CompactString};

use super::{CorsaTypeAwareSession, LintResult, Linter, RULE_NO_FLOATING_PROMISES, push_warning};
use crate::diagnostic::LintDiagnostic;
use crate::linter::corsa_session::{
    ExpressionFact, ExpressionQuery, PredicateRequest as ShapeCall,
};
use corsa::lint::LintNode;
use facts::{Wants, queries};
use names::{RULES, corsa_name, static_rule};
use serde_json::Value;

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
    let floating = floating_active(linter);
    let options = &linter.type_aware_options;
    let wants = Wants::new(
        &corsa_rules,
        enabled.contains(&"type/no-generated-empty-object-type"),
        floating,
        checks_type_predicates(options),
    );
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
    listeners.insert("VariableDeclarator");
    if wants.empty_object() {
        listeners.extend(gaps::EMPTY_OBJECT_LISTENERS);
    }
    if floating {
        listeners.insert("ExpressionStatement");
    }
    if wants.promise().any() {
        // Promise executors are found from their `new` expression, and object
        // literal methods from their object's contextual type.
        listeners.insert("NewExpression");
        listeners.insert("ObjectExpression");
    }
    let listener_list = listeners.iter().copied().collect::<Vec<_>>();
    for script in &plan.scripts {
        let map = |(start, end): (u32, u32)| {
            generated_range(&plan.mapping, virtual_content, script, start, end)
        };
        let (mut collected, pending) = tree::collect_with(
            &script.content,
            script.tsx,
            &listener_list,
            |collected, ast| relate::plan(ast, &collected.program, &corsa_rules, &map),
        );
        facts::prepare(&script.content, &mut collected);
        let script_queries = queries(&collected.nodes, wants);
        let mut generated = Vec::new();
        let mut used = Vec::new();
        for (index, query) in script_queries.iter().enumerate() {
            let Some((start, end)) = map((query.start, query.end)) else {
                continue;
            };
            let shape = query.shape.clone().map(|mut request| {
                request.keys_from = request.keys_from.and_then(map);
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
            });
            generated.push(ExpressionQuery {
                start,
                end,
                shape,
                ..query.clone()
            });
            used.push(index);
        }
        let request = (!pending.is_empty()).then(|| pending.request());
        let (probed, relations) = session.expression_facts(virtual_content, &generated, request)?;
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
        if !pending.is_empty() {
            relate::apply(&mut collected.nodes, &pending, relations.as_ref());
        }
        let mut seen = FxHashSet::default();
        if floating {
            let configured = options::for_rule(options, "no-floating-promises");
            for node in collected
                .nodes
                .iter()
                .filter(|node| node.kind == "ExpressionStatement")
            {
                for diagnostic in floating_reports(node, configured.as_ref()) {
                    let start = script.offset.saturating_add(diagnostic.range.start);
                    let end = script.offset.saturating_add(diagnostic.range.end);
                    // typescript-eslint decides; the report keeps vize's own
                    // message, as on plain modules and in templates.
                    if seen.insert((
                        RULE_NO_FLOATING_PROMISES,
                        start,
                        end,
                        FLOATING_MESSAGE.into(),
                    )) {
                        push_warning(
                            result,
                            LintDiagnostic::warn(
                                RULE_NO_FLOATING_PROMISES,
                                FLOATING_MESSAGE,
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
                let mut prepared = present::prepare(rule, node);
                if let Some(configured) = options::for_rule(options, rule) {
                    prepared.fields.insert("__ruleOptions".into(), configured);
                }
                for diagnostic in
                    run_default_type_aware_rule_owned(rule, prepared).unwrap_or_default()
                {
                    let (raw_start, raw_end) =
                        present::range(rule, node, diagnostic.range.start, diagnostic.range.end);
                    let message = present::message(
                        rule,
                        node,
                        diagnostic.range.start,
                        diagnostic.message.into(),
                    );
                    let start = script.offset.saturating_add(raw_start);
                    let end = script.offset.saturating_add(raw_end);
                    if seen.insert((vize_name, start, end, message.clone())) {
                        push_warning(result, LintDiagnostic::warn(vize_name, message, start, end));
                    }
                }
            }
        }
        for gap in gaps::reports(&collected) {
            let start = script.offset.saturating_add(gap.start);
            let end = script.offset.saturating_add(gap.end);
            if seen.insert((gap.rule, start, end, gap.message.into())) {
                push_warning(
                    result,
                    LintDiagnostic::warn(gap.rule, gap.message, start, end),
                );
            }
        }
    }
    Ok(())
}

/// vize's `type/no-floating-promises` message.
const FLOATING_MESSAGE: &str =
    "Floating Promise must be awaited, returned, or explicitly ignored with `void`";

/// typescript-eslint's no-floating-promises on one statement, with the
/// configured `linter.ruleOptions`. Upstream reports the statement; vize marks
/// its expression.
fn floating_reports(
    node: &LintNode,
    configured: Option<&Value>,
) -> Vec<corsa::lint::LintDiagnostic> {
    let Some(expression) = node.child("expression") else {
        return Vec::new();
    };
    let mut prepared = node.clone();
    if let Some(configured) = configured {
        prepared
            .fields
            .insert("__ruleOptions".into(), configured.clone());
    }
    let mut diagnostics =
        run_default_type_aware_rule_owned("no-floating-promises", prepared).unwrap_or_default();
    for diagnostic in &mut diagnostics {
        diagnostic.range = expression.range;
    }
    diagnostics
}

/// The generated range that copies script range `start..end` verbatim.
///
/// Rows can link the same authored bytes twice: a macro name also maps to the
/// prop key its declaration emits, so `defineModel` links to `"modelValue"` in
/// the props type ahead of the setup statement that copies it. A caret lookup
/// takes the first such row, and mapping a range's two ends one at a time can
/// start in the props type and end in setup. Only one row that holds the whole
/// range, and whose generated bytes equal the authored ones, answers for it.
/// no-unnecessary-condition's configured `checkTypePredicates`, which decides
/// whether type predicates are worth their checker requests.
fn checks_type_predicates(options: &vize_l0::config::TypeAwareRuleOptions) -> bool {
    options::for_rule(options, "no-unnecessary-condition")
        .and_then(|list| list.get(0)?.get("checkTypePredicates")?.as_bool())
        .unwrap_or(false)
}

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
    let verbatim = |span: &VizeMapping| {
        let generated_start = span
            .gen_range
            .start
            .checked_add(authored_start.checked_sub(span.src_range.start)?)?;
        let generated_end = generated_start.checked_add(authored.len())?;
        (generated.get(generated_start..generated_end)? == authored).then_some((
            u32::try_from(generated_start).ok()?,
            u32::try_from(generated_end).ok()?,
        ))
    };
    let holds = |span: &&VizeMapping, end: usize| {
        span.src_range.start <= authored_start && end <= span.src_range.end
    };
    // A row that holds the whole range answers first. A range that crosses
    // rows (an expression whose callback the projection emits as rows of its
    // own) falls back to a row holding its start, still only where the
    // generated bytes equal the authored ones.
    mapping
        .spans()
        .iter()
        .filter(|span| holds(span, authored_end))
        .find_map(verbatim)
        .or_else(|| {
            mapping
                .spans()
                .iter()
                .filter(|span| holds(span, authored_start.saturating_add(1)))
                .find_map(verbatim)
        })
}
