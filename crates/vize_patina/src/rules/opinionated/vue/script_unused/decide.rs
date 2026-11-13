//! Turn script bindings plus template reads and writes into the two reports.

use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;
use vize_l0::{CompactString, FxHashMap, FxHashSet, String, cstr};

use super::classify::{BindingInfo, Classified, Kind, ParamSlot, classify};
use super::options::{
    Args, Caught, Destructuring, Pattern, PreferConstOptions, UnusedOptions, Vars, allowance,
    used_allowance,
};
use super::template_use::TemplateUse;
use super::usage::{Facts, Usage};

pub(super) enum Which {
    Unused,
    PreferConst,
}

pub(super) struct Finding {
    pub(super) start: u32,
    pub(super) end: u32,
    pub(super) message: String,
}

/// What the script and the template do with one binding.
struct Seen {
    facts: Facts,
    template_read: bool,
    template_write: bool,
}

impl Seen {
    fn used(&self) -> bool {
        self.facts.used || self.template_read
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "one script block with its template, sibling-read and rule options"
)]
pub(super) fn analyze(
    program: &oxc_ast::ast::Program<'_>,
    offset: u32,
    template: &TemplateUse,
    sibling_reads: &FxHashSet<CompactString>,
    unused: &UnusedOptions,
    prefer: &PreferConstOptions,
    which: Which,
) -> Vec<Finding> {
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(program)
        .semantic;
    let scoping = semantic.scoping();
    let usage = Usage::new(scoping, semantic.nodes());
    let classified = classify(program);
    let seen: FxHashMap<SymbolId, Seen> = classified
        .infos
        .iter()
        .map(|(id, info)| {
            let module_binding = scoping.symbol_scope_id(*id) == scoping.root_scope_id();
            let name = scoping.symbol_name(*id);
            let seen = Seen {
                facts: usage.facts(*id, info, &classified.for_return_targets),
                // A template cannot reassign a `const`. Vue compiles
                // `hovered = $event` on one to a write through the ref's
                // `.value`, which reads the binding. A `let` write stays an
                // assignment. A root binding of either script block is also
                // read when the other block names it: both blocks are one module.
                template_read: module_binding
                    && (template.reads.contains(name)
                        || (info.kind == Kind::Const && template.writes.contains(name))
                        || sibling_reads.contains(name)),
                template_write: module_binding && template.writes.contains(name),
            };
            (*id, seen)
        })
        .collect();
    let mut findings = Vec::new();
    match which {
        Which::Unused => unused_findings(&classified, &seen, scoping, unused, &mut findings),
        Which::PreferConst => {
            prefer_findings(&classified, &seen, scoping, &usage, prefer, &mut findings);
        }
    }
    let mut findings: Vec<Finding> = findings
        .into_iter()
        .filter_map(|(span, message)| {
            let start = offset.saturating_add(span.start);
            let end = offset.saturating_add(span.end);
            (end > start).then_some(Finding {
                start,
                end,
                message,
            })
        })
        .collect();
    findings.sort_by_key(|finding| (finding.start, finding.end));
    findings
}

fn unused_findings(
    classified: &Classified,
    seen: &FxHashMap<SymbolId, Seen>,
    scoping: &Scoping,
    options: &UnusedOptions,
    findings: &mut Vec<(Span, String)>,
) {
    let mut ignored = FxHashSet::default();
    if options.ignore_rest_siblings {
        for group in &classified.rests {
            let rest_used = group.rest.iter().any(|id| {
                classified.infos.get(id).is_some_and(|info| info.exported)
                    || seen.get(id).is_some_and(Seen::used)
            });
            if rest_used {
                ignored.extend(group.siblings.iter().copied());
            }
        }
    }
    let params = param_reports(&classified.functions, seen, options.args);
    for (id, info) in &classified.infos {
        let Some(seen) = seen.get(id) else {
            continue;
        };
        if info.exported {
            continue;
        }
        let name = scoping.symbol_name(*id);
        let (pattern, kind) = ignore_pattern(info, options);
        let array_pattern = options.array_ignore.as_ref().filter(|_| info.array_element);
        let by_kind = pattern.filter(|pattern| pattern.matches(name));
        let by_array = array_pattern.filter(|pattern| pattern.matches(name));
        if seen.used() {
            if options.report_used_ignore_pattern {
                let matched = by_array
                    .map(|pattern| (pattern, ARRAY_ELEMENTS))
                    .or(by_kind.map(|pattern| (pattern, kind)));
                if let Some((pattern, kind)) = matched {
                    let suffix = used_allowance(kind, pattern);
                    let message = cstr!("'{name}' is marked as ignored but is used.{suffix}");
                    findings.push((scoping.symbol_span(*id), message));
                }
            }
            continue;
        }
        let skipped = by_kind.is_some()
            || by_array.is_some()
            || ignored.contains(id)
            || match info.kind {
                Kind::Param => !params.contains(id),
                Kind::Catch => matches!(options.caught, Caught::None),
                Kind::Class => info.static_block && options.ignore_static_block_classes,
                _ => {
                    matches!(options.vars, Vars::Local)
                        && scoping.symbol_scope_id(*id) == scoping.root_scope_id()
                }
            };
        if skipped {
            continue;
        }
        let assigned = info.has_init || seen.facts.any_write || seen.template_write;
        let verb = if assigned {
            "is assigned a value but never used."
        } else {
            "is defined but never used."
        };
        let suffix = match array_pattern {
            Some(array) if assigned && !matches!(info.kind, Kind::Param | Kind::Catch) => {
                allowance(ARRAY_ELEMENTS, Some(array))
            }
            _ => allowance(kind, pattern),
        };
        let span = match (info.init_span, seen.facts.last_local_write) {
            (Some(init), Some(write)) if init.start > write.start => init,
            (_, Some(write)) => write,
            (Some(init), None) => init,
            (None, None) => scoping.symbol_span(*id),
        };
        findings.push((span, cstr!("'{name}' {verb}{suffix}")));
    }
}

const ARRAY_ELEMENTS: &str = "elements of array destructuring";

/// The pattern for the binding's kind, and its wording in messages.
fn ignore_pattern<'o>(
    info: &BindingInfo,
    options: &'o UnusedOptions,
) -> (Option<&'o Pattern>, &'static str) {
    match info.kind {
        Kind::Param => (options.args_ignore.as_ref(), "args"),
        Kind::Catch => (options.caught_ignore.as_ref(), "caught errors"),
        _ => (options.vars_ignore.as_ref(), "vars"),
    }
}

fn prefer_findings(
    classified: &Classified,
    seen: &FxHashMap<SymbolId, Seen>,
    scoping: &Scoping,
    usage: &Usage<'_, '_>,
    options: &PreferConstOptions,
    findings: &mut Vec<(Span, String)>,
) {
    let mut targets = FxHashMap::default();
    let mut blocked_groups = FxHashSet::default();
    for (id, info) in &classified.infos {
        if !matches!(info.kind, Kind::Let | Kind::Var) || info.exported {
            continue;
        }
        let template_write = seen.get(id).is_some_and(|seen| seen.template_write);
        let target = if template_write {
            None
        } else {
            usage.const_target(*id, info, options.ignore_read_before_assign)
        };
        match target {
            Some(span) => {
                targets.insert(*id, span);
            }
            None if info.group != 0 => {
                blocked_groups.insert(info.group);
            }
            None => {}
        }
    }
    for (id, span) in targets {
        let Some(info) = classified.infos.get(&id) else {
            continue;
        };
        if matches!(options.destructuring, Destructuring::All)
            && blocked_groups.contains(&info.group)
        {
            continue;
        }
        let name = scoping.symbol_name(id);
        findings.push((
            span,
            cstr!("'{name}' is never reassigned. Use 'const' instead"),
        ));
    }
}

/// Parameters the `args` setting checks. ESLint `after-used` skips a plain
/// parameter only when a later parameter is referenced; destructured,
/// defaulted, and rest parameters are always checked.
fn param_reports(
    functions: &[Vec<ParamSlot>],
    seen: &FxHashMap<SymbolId, Seen>,
    args: Args,
) -> FxHashSet<SymbolId> {
    let mut reports = FxHashSet::default();
    if matches!(args, Args::None) {
        return reports;
    }
    let referenced = |id: &SymbolId| {
        seen.get(id)
            .is_some_and(|seen| seen.facts.any_reference || seen.template_read)
    };
    for slots in functions {
        for (index, slot) in slots.iter().enumerate() {
            let include = match args {
                Args::All | Args::None => true,
                Args::AfterUsed => {
                    !slot.plain
                        || !slots
                            .iter()
                            .skip(index + 1)
                            .any(|later| later.ids.iter().any(referenced))
                }
            };
            if include {
                reports.extend(slot.ids.iter().copied());
            }
        }
    }
    reports
}
