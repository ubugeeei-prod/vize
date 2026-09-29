//! Type-aware rules for a plain TypeScript module.
//!
//! SFC-only rules stay on the descriptor path. `type/no-floating-promises` and
//! `type/no-reactivity-loss` also run here, because composables live in `.ts`.

use vize_canon::virtual_ts::ProjectionMapping;
use vize_croquis::script_parser::{self, ScriptParseResult, ScriptParserOptions};
use vize_l0::{FxHashSet, String, ToCompactString, profile};

use super::document::TypeAwareDocument;
use super::parsing::collect_floating_candidates_from;
use super::reactivity_loss::{ReactivityLossQuery, collect_reactivity_loss_queries};
use super::{
    LintResult, Linter, RULE_NO_FLOATING_PROMISES, RULE_NO_REACTIVITY_LOSS, TypeProbe,
    has_promise_like_return, push_warning, should_warn_for_reactivity_loss, with_corsa_session,
};
use crate::diagnostic::LintDiagnostic;

mod syntax;

const PLAIN_HOST: &str = "function __vize_plain_host() {\n\n}\n\n// Invoke setup to verify types\n";

pub(in crate::linter) fn append_plain_module_type_diagnostics(
    linter: &Linter,
    source: &str,
    filename: &str,
    result: &mut LintResult,
) {
    if !linter.type_aware_enabled || !is_plain_type_script(filename) {
        return;
    }
    let promises = rule_active(linter, RULE_NO_FLOATING_PROMISES);
    let reactivity = rule_active(linter, RULE_NO_REACTIVITY_LOSS);
    if !promises && !reactivity {
        return;
    }

    let mut slots = Vec::new();
    if promises {
        push_promise_slots(source, filename, &mut slots);
    }
    if reactivity {
        push_reactivity_slots(linter, source, filename, result, &mut slots);
    }
    place_slots_after_statements(source, filename, &mut slots);
    if slots.is_empty() {
        super::super::severity::apply_severity_overrides(result, &linter.severity_overrides);
        return;
    }

    let (augmented, probes) = augment_source(source, &slots);
    let corsa_result = profile!("patina.type_aware.plain_module", {
        with_corsa_session(linter, filename, |session| {
            session.open_virtual_project(augmented.as_str(), filename)?;
            let mut seen_promises = FxHashSet::default();
            let mut seen_reactivity = FxHashSet::default();
            for (offset, kind) in &probes {
                if !remember_probe(kind, &mut seen_promises, &mut seen_reactivity) {
                    continue;
                }
                let probe = session.probe_type_at_offset(
                    augmented.as_str(),
                    *offset,
                    false,
                    kind.loads_signatures(),
                )?;
                report_probe(result, kind, probe.as_ref());
            }
            Ok(())
        })
    });
    if let Err(error) = corsa_result {
        push_warning(
            result,
            LintDiagnostic::warn("type/corsa-runtime", error, 0, 0).with_help(
                "Type-aware lint rules were skipped because the Corsa runtime could not be started. Configure `typeChecker.corsaPath` or install `typescript@^7`.",
            ),
        );
    }
    super::super::severity::apply_severity_overrides(result, &linter.severity_overrides);
}

fn push_promise_slots(source: &str, filename: &str, slots: &mut Vec<Slot>) {
    for (index, candidate) in collect_floating_candidates_from(source, filename)
        .into_iter()
        .enumerate()
    {
        let Some(expression) = source_slice(source, candidate.start, candidate.end) else {
            continue;
        };
        let (text, name_rel) = promise_marker(index, expression);
        slots.push(Slot {
            at: candidate.end,
            range_start: candidate.start,
            text,
            name_rel,
            kind: Probe::Promise {
                start: candidate.start,
                end: candidate.end,
            },
        });
    }
}

fn push_reactivity_slots(
    linter: &Linter,
    source: &str,
    filename: &str,
    result: &mut LintResult,
    slots: &mut Vec<Slot>,
) {
    let mut host = TypeAwareDocument {
        content: PLAIN_HOST.to_compact_string(),
        mapping: ProjectionMapping::new(),
    };
    let parsed = parse_plain_module(source, filename);
    for (index, query) in
        collect_reactivity_loss_queries(linter, result, &parsed, source, 0, &mut host)
            .into_iter()
            .enumerate()
    {
        let Some(expression) = source_slice(source, query.source_start, query.source_end) else {
            continue;
        };
        let (text, name_rel) = reactivity_marker(index, &query, expression);
        slots.push(Slot {
            at: query.source_end,
            range_start: query.source_start,
            text,
            name_rel,
            kind: Probe::Reactivity(query),
        });
    }
}

fn place_slots_after_statements(source: &str, filename: &str, slots: &mut Vec<Slot>) {
    let spans = syntax::statement_spans(source, filename);
    let mut placed = Vec::with_capacity(slots.len());
    for mut slot in slots.drain(..) {
        let Some(end) = syntax::enclosing_statement_end(&spans, slot.range_start, slot.at) else {
            continue;
        };
        slot.at = end;
        placed.push(slot);
    }
    *slots = placed;
}

fn report_probe(result: &mut LintResult, kind: &Probe, probe: Option<&TypeProbe>) {
    match kind {
        Probe::Promise { start, end } => {
            let Some(probe) = probe else {
                return;
            };
            if has_promise_like_return(probe)
                || corsa::utils::is_promise_like_type_texts(
                    &probe.type_texts,
                    &probe.property_names,
                )
            {
                push_warning(
                    result,
                    LintDiagnostic::warn(
                        RULE_NO_FLOATING_PROMISES,
                        "Floating Promise must be awaited, returned, or explicitly ignored with `void`",
                        *start,
                        *end,
                    )
                    .with_help(
                        "Add `await`, return the Promise, or prefix it with `void` when the fire-and-forget behavior is intentional.",
                    ),
                );
            }
        }
        Probe::Reactivity(query) => {
            if should_warn_for_reactivity_loss(probe) {
                push_warning(result, query.diagnostic(0));
            }
        }
    }
}

fn remember_probe(
    kind: &Probe,
    promises: &mut FxHashSet<(u32, u32)>,
    reactivity: &mut FxHashSet<u64>,
) -> bool {
    match kind {
        Probe::Promise { start, end } => promises.insert((*start, *end)),
        Probe::Reactivity(query) => reactivity.insert(query.owner_key()),
    }
}

struct Slot {
    at: u32,
    range_start: u32,
    text: String,
    name_rel: u32,
    kind: Probe,
}

#[derive(Clone)]
enum Probe {
    Promise { start: u32, end: u32 },
    Reactivity(ReactivityLossQuery),
}

impl Probe {
    fn loads_signatures(&self) -> bool {
        matches!(self, Self::Promise { .. })
    }
}

fn augment_source(source: &str, slots: &[Slot]) -> (String, Vec<(u32, Probe)>) {
    let mut order: Vec<(u32, usize)> = slots
        .iter()
        .enumerate()
        .map(|(index, slot)| (slot.at, index))
        .collect();
    order.sort_unstable();
    let extra: usize = slots.iter().map(|slot| slot.text.len()).sum();
    let mut out = String::with_capacity(source.len() + extra);
    let mut probes = Vec::with_capacity(slots.len());
    let mut cursor = 0usize;
    for (_, index) in order {
        let Some(slot) = slots.get(index) else {
            continue;
        };
        let at = slot.at as usize;
        if at < cursor || at > source.len() {
            continue;
        }
        let Some(chunk) = source.get(cursor..at) else {
            continue;
        };
        out.push_str(chunk);
        cursor = at;
        let Some(offset) = u32::try_from(out.len() + slot.name_rel as usize).ok() else {
            continue;
        };
        probes.push((offset, slot.kind.clone()));
        out.push_str(slot.text.as_str());
    }
    if let Some(rest) = source.get(cursor..) {
        out.push_str(rest);
    }
    (out, probes)
}

fn promise_marker(index: usize, expression: &str) -> (String, u32) {
    let mut name = String::with_capacity(32);
    name.push_str("__vize_patina_promise_");
    name.push_str(index.to_compact_string().as_str());
    marker_text(&name, expression, "function ", "() { return (", "); }\n")
}

fn reactivity_marker(index: usize, query: &ReactivityLossQuery, expression: &str) -> (String, u32) {
    let mut name = String::with_capacity(48);
    name.push_str("__vize_patina_reactivity_");
    name.push_str(query.source_start.to_compact_string().as_str());
    name.push('_');
    name.push_str(query.source_end.to_compact_string().as_str());
    name.push('_');
    name.push_str(index.to_compact_string().as_str());
    marker_text(&name, expression, "const ", " = (", ");\n")
}

fn marker_text(
    name: &str,
    expression: &str,
    before: &str,
    between: &str,
    after: &str,
) -> (String, u32) {
    let mut text = String::with_capacity(
        name.len() + expression.len() + before.len() + between.len() + after.len() + 1,
    );
    text.push('\n');
    text.push_str(before);
    let name_rel = text.len() as u32;
    text.push_str(name);
    text.push_str(between);
    text.push_str(expression);
    text.push_str(after);
    (text, name_rel)
}

fn parse_plain_module(source: &str, filename: &str) -> ScriptParseResult {
    let options = ScriptParserOptions {
        options_api: true,
        legacy_vue2: super::script_options::is_likely_legacy_vue2_script(source),
    };
    if is_tsx(filename) {
        script_parser::parse_script_with_options_and_jsx(source, options, true)
    } else {
        script_parser::parse_script_with_options(source, options)
    }
}

fn source_slice(source: &str, start: u32, end: u32) -> Option<&str> {
    source
        .get(start as usize..end as usize)
        .map(str::trim)
        .filter(|expression| !expression.is_empty())
}

fn rule_active(linter: &Linter, rule_name: &str) -> bool {
    linter.registry.has_rule(rule_name) && linter.is_rule_enabled(rule_name)
}

fn is_plain_type_script(filename: &str) -> bool {
    let name = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    if name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts") {
        return false;
    }
    matches!(
        name.rsplit_once('.'),
        Some((_, "ts" | "mts" | "cts" | "tsx"))
    )
}

fn is_tsx(filename: &str) -> bool {
    let name = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    matches!(name.rsplit_once('.'), Some((_, "tsx")))
}
