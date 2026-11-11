//! Turn script bindings plus template reads and writes into the two reports.

use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_syntax::symbol::SymbolFlags;
use vize_l0::FxHashSet;

use super::classify::{BindingInfo, Kind, classify};
use super::options::{Caught, Destructuring, UnusedOptions, Vars, allowance};
use super::template_use::TemplateUse;

pub(super) enum Which {
    Unused,
    PreferConst,
}

pub(super) struct Finding {
    pub(super) start: u32,
    pub(super) end: u32,
    pub(super) message: String,
}

pub(super) fn analyze(
    program: &oxc_ast::ast::Program<'_>,
    offset: u32,
    template: &TemplateUse,
    apply_template: bool,
    unused: &UnusedOptions,
    prefer: &super::options::PreferConstOptions,
    which: Which,
) -> Vec<Finding> {
    let semantic = SemanticBuilder::new().build(program).semantic;
    let scoping = semantic.scoping();
    let classified = classify(program);
    let mut ignored = FxHashSet::default();
    if unused.ignore_rest_siblings {
        for group in &classified.rests {
            let rest_used = group.rest.iter().any(|id| {
                classified.infos.get(id).is_some_and(|info| info.exported)
                    || symbol_used(scoping, *id, template, apply_template)
            });
            if rest_used {
                ignored.extend(group.siblings.iter().copied());
            }
        }
    }
    let params = param_reports(
        &classified.functions,
        scoping,
        template,
        apply_template,
        unused,
    );
    let suppressed =
        suppressed_groups(&classified.infos, scoping, template, apply_template, prefer);
    let mut findings = Vec::new();
    for (id, info) in &classified.infos {
        if info.exported || scoping.symbol_flags(*id).contains(SymbolFlags::Ambient) {
            continue;
        }
        let name = scoping.symbol_name(*id);
        let used = symbol_used(scoping, *id, template, apply_template);
        let mutated = symbol_mutated(scoping, *id, template, apply_template);
        let span = scoping.symbol_span(*id);
        let start = offset.saturating_add(span.start);
        let end = offset.saturating_add(span.end);
        if end <= start {
            continue;
        }
        let message = match which {
            Which::Unused => {
                unused_message(name, info, used, &params, &ignored, unused, scoping, *id)
            }
            Which::PreferConst => prefer_message(name, info, used, mutated, &suppressed, prefer),
        };
        if let Some(message) = message {
            findings.push(Finding {
                start,
                end,
                message,
            });
        }
    }
    findings.sort_by_key(|finding| (finding.start, finding.end));
    findings
}

fn unused_message(
    name: &str,
    info: &BindingInfo,
    used: bool,
    params: &FxHashSet<oxc_syntax::symbol::SymbolId>,
    ignored: &FxHashSet<oxc_syntax::symbol::SymbolId>,
    options: &UnusedOptions,
    scoping: &Scoping,
    id: oxc_syntax::symbol::SymbolId,
) -> Option<String> {
    if used {
        return None;
    }
    let (allowance, report) = match info.kind {
        Kind::Param => (
            allowance("args", options.args_ignore.as_ref()),
            params.contains(&id) && !matches_pattern(options.args_ignore.as_ref(), name),
        ),
        Kind::Catch => (
            allowance("caught errors", options.caught_ignore.as_ref()),
            matches!(options.caught, Caught::All)
                && !matches_pattern(options.caught_ignore.as_ref(), name),
        ),
        _ => {
            let local_only = matches!(options.vars, Vars::Local)
                && scoping.symbol_scope_id(id) == scoping.root_scope_id();
            let skipped = local_only
                || ignored.contains(&id)
                || matches_pattern(options.vars_ignore.as_ref(), name)
                || (info.array_element && matches_pattern(options.array_ignore.as_ref(), name));
            (allowance("vars", options.vars_ignore.as_ref()), !skipped)
        }
    };
    if !report {
        return None;
    }
    let assigned = info.has_init && matches!(info.kind, Kind::Let | Kind::Const | Kind::Var);
    let verb = if assigned {
        "is assigned a value but never used."
    } else {
        "is defined but never used."
    };
    Some(format!("'{name}' {verb}{allowance}"))
}

fn prefer_message(
    name: &str,
    info: &BindingInfo,
    used: bool,
    mutated: bool,
    suppressed: &FxHashSet<u32>,
    options: &super::options::PreferConstOptions,
) -> Option<String> {
    if mutated || !matches!(info.kind, Kind::Let | Kind::Var) {
        return None;
    }
    if options.ignore_read_before_assign && !info.has_init && used {
        return None;
    }
    if matches!(options.destructuring, Destructuring::All)
        && info.group != 0
        && suppressed.contains(&info.group)
    {
        return None;
    }
    Some(format!("'{name}' is never reassigned. Use 'const' instead"))
}

fn param_reports(
    functions: &[Vec<Vec<oxc_syntax::symbol::SymbolId>>],
    scoping: &Scoping,
    template: &TemplateUse,
    apply_template: bool,
    options: &UnusedOptions,
) -> FxHashSet<oxc_syntax::symbol::SymbolId> {
    let mut reports = FxHashSet::default();
    if matches!(options.args, super::options::Args::None) {
        return reports;
    }
    for slots in functions {
        let mut last_used = None;
        for (index, slot) in slots.iter().enumerate() {
            if slot
                .iter()
                .any(|id| symbol_used(scoping, *id, template, apply_template))
            {
                last_used = Some(index);
            }
        }
        for (index, slot) in slots.iter().enumerate() {
            let include = match options.args {
                super::options::Args::All => true,
                super::options::Args::None => false,
                super::options::Args::AfterUsed => {
                    last_used.map(|last| index > last).unwrap_or(true)
                }
            };
            if include {
                reports.extend(slot.iter().copied());
            }
        }
    }
    reports
}

fn suppressed_groups(
    infos: &vize_l0::FxHashMap<oxc_syntax::symbol::SymbolId, BindingInfo>,
    scoping: &Scoping,
    template: &TemplateUse,
    apply_template: bool,
    options: &super::options::PreferConstOptions,
) -> FxHashSet<u32> {
    let mut groups = FxHashSet::default();
    if !matches!(options.destructuring, Destructuring::All) {
        return groups;
    }
    for (id, info) in infos {
        if info.group != 0
            && matches!(info.kind, Kind::Let | Kind::Var)
            && symbol_mutated(scoping, *id, template, apply_template)
        {
            groups.insert(info.group);
        }
    }
    groups
}

fn symbol_used(
    scoping: &Scoping,
    id: oxc_syntax::symbol::SymbolId,
    template: &TemplateUse,
    apply: bool,
) -> bool {
    let script = scoping.get_resolved_references(id).any(|reference| {
        let flags = reference.flags();
        flags.is_read() || flags.is_type() || flags.is_value_as_type()
    });
    script
        || (apply
            && scoping.symbol_scope_id(id) == scoping.root_scope_id()
            && template.reads.contains(scoping.symbol_name(id)))
}

fn symbol_mutated(
    scoping: &Scoping,
    id: oxc_syntax::symbol::SymbolId,
    template: &TemplateUse,
    apply: bool,
) -> bool {
    if scoping
        .symbol_flags(id)
        .contains(SymbolFlags::ConstVariable)
    {
        return false;
    }
    scoping.symbol_is_mutated(id)
        || (apply
            && scoping.symbol_scope_id(id) == scoping.root_scope_id()
            && template.writes.contains(scoping.symbol_name(id)))
}

fn matches_pattern(pattern: Option<&super::options::Pattern>, name: &str) -> bool {
    pattern.is_some_and(|pattern| pattern.matches(name))
}
