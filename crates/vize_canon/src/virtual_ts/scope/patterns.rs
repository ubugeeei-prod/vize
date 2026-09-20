//! Typed RFC 823 scopes shared by expression and component-prop generation.

use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
mod descriptor;
mod source;

use vize_armature::patterns::{MatchPattern, PatternKind};
use vize_carton::{FxHashMap, FxHashSet, String, append, cstr};
use vize_croquis::{Croquis, Scope, ScopeData, ScopeId};

use crate::virtual_ts::types::VizeMapping;
use descriptor::{emit_bindings, emit_descriptor};
use source::{PatternEmitter, map_range};

pub(super) fn generate_expression_match(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &super::context::ScopeGenContext<'_, '_>,
    scope: &Scope,
    indent: &str,
) {
    let patterns = PatternContext {
        summary: ctx.summary,
        children: ctx.children_map,
        template_source: ctx.template_source,
        template_offset: ctx.template_offset,
        template_binding_access: ctx.template_binding_access,
        verification: ctx.check_options.check_template_bindings,
    };
    emit_match(
        ts,
        mappings,
        &patterns,
        scope,
        indent,
        |ts, mappings, scope, indent| {
            if matches!(scope.data(), ScopeData::VMatch(_)) {
                super::node::generate_scope_contents(ts, mappings, ctx, scope, indent);
            } else {
                super::closures::generate_scope_node(ts, mappings, ctx, scope, indent);
            }
        },
    );
}

pub(super) fn generate_props_match(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &super::context::VForPropsContext<'_, '_>,
    scope: &Scope,
    indent: &str,
) {
    let patterns = PatternContext {
        summary: ctx.summary,
        children: ctx.children_map,
        template_source: ctx.source_context.template,
        template_offset: ctx.source_context.offset,
        template_binding_access: ctx.template_binding_access,
        verification: false,
    };
    emit_match(
        ts,
        mappings,
        &patterns,
        scope,
        indent,
        |ts, mappings, scope, indent| {
            if matches!(scope.data(), ScopeData::VMatch(_)) {
                super::empty_component_props::generate_scope_checks(
                    ts,
                    mappings,
                    ctx,
                    scope.id.as_u32(),
                    indent,
                );
            } else {
                super::component_props::generate_closure_component_props_recursive(
                    ts, mappings, ctx, scope, indent,
                );
            }
        },
    );
}

pub(super) struct PatternContext<'a> {
    pub summary: &'a Croquis,
    pub children: &'a FxHashMap<u32, Vec<ScopeId>>,
    pub template_source: Option<&'a str>,
    pub template_offset: u32,
    pub template_binding_access: &'a TemplateBindingAccess,
    pub verification: bool,
}

/// Replay exactly the same arm environment for both code-generation passes.
pub(super) fn emit_match(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &PatternContext<'_>,
    scope: &Scope,
    indent: &str,
    mut body: impl FnMut(&mut String, &mut Vec<VizeMapping>, &Scope, &str),
) {
    let ScopeData::VMatch(subject) = scope.data() else {
        return;
    };
    let mut prefix = cstr!("__vize_match_{}_", scope.id.as_u32());
    // Include script and template spellings, not only known bindings: unresolved
    // references must not accidentally start resolving to a generated local.
    while ts.contains(prefix.as_str())
        || ctx
            .template_source
            .is_some_and(|source| source.contains(prefix.as_str()))
    {
        prefix.push('_');
    }
    let inner = cstr!("{indent}  ");
    let mut emitter = PatternEmitter::new(ctx, prefix, &inner);
    let local = emitter.name();
    let guard = ctx
        .summary
        .template_expressions
        .iter()
        .find(|expression| expression.start == subject.start && expression.end == subject.end)
        .and_then(|expression| expression.vif_guard.as_deref());
    if let Some(guard) = guard {
        super::vif_guard::append_ignored_vif_guard_open(
            ts,
            indent,
            guard,
            "Enclosing pattern guard",
        );
    }
    append!(*ts, "{indent}{{\n{inner}const {local} = (");
    let subject_text =
        emitter.expression(ts, mappings, &subject.subject, subject.start, subject.end);
    ts.push_str(");\n");
    body(ts, mappings, scope, &inner);
    // Capture before a predicate narrows `local`: the second predicate must
    // still accept the unrefined authored subject.
    let subject_type = emitter.name();
    append!(*ts, "{inner}type {subject_type} = typeof {local};\n");
    let mut remaining = subject_type.clone();
    if let Some(children) = ctx.children.get(&scope.id.as_u32()) {
        for id in children {
            let Some(arm_scope) = ctx.summary.scopes.get_scope(*id) else {
                continue;
            };
            let ScopeData::VWhen(when) = arm_scope.data() else {
                body(ts, mappings, arm_scope, &inner);
                continue;
            };
            let descriptor =
                emit_descriptor(&mut emitter, ts, mappings, &when.arm.pattern, when.offset);
            let pattern_type = emitter.name();
            let narrowed = emitter.name();
            append!(
                *ts,
                "{inner}type {pattern_type} = {descriptor};\n{inner}type {narrowed} = __VizePatterns.Match<{remaining}, {pattern_type}>;\n"
            );
            if ctx.verification {
                let marker = cstr!("{}unreachable", emitter.name());
                append!(*ts, "{inner}const ");
                let start = ts.len();
                ts.push_str(&marker);
                map_range(
                    mappings,
                    start..ts.len(),
                    (ctx.template_offset + when.offset + when.arm.pattern.span.start) as usize
                        ..(ctx.template_offset + when.offset + when.arm.pattern.span.end) as usize,
                );
                append!(
                    *ts,
                    ": __VizePatterns.Reachable<{narrowed}> = true;\n{inner}void {marker};\n"
                );
            }
            let condition = cstr!(
                "((value: {subject_type}): value is {subject_type} & {narrowed} => (void value, true))"
            );
            append!(
                *ts,
                "{inner}if ({condition}({local}) && {condition}(({subject_text}))) {{\n"
            );
            let arm_indent = cstr!("{inner}  ");
            let value = emitter.name();
            append!(*ts, "{arm_indent}const {value} = {local} as {narrowed};\n");
            emit_bindings(
                &mut emitter,
                ts,
                mappings,
                &when.arm.pattern,
                &value,
                when.offset,
                &arm_indent,
            );
            if let Some(guard) = &when.arm.guard {
                append!(*ts, "{arm_indent}if (");
                emitter.expression(
                    ts,
                    mappings,
                    &guard.text,
                    when.offset + guard.span.start,
                    when.offset + guard.span.end,
                );
                ts.push_str(") {\n");
                body(ts, mappings, arm_scope, &cstr!("{arm_indent}  "));
                append!(*ts, "{arm_indent}}}\n");
            } else {
                body(ts, mappings, arm_scope, &arm_indent);
            }
            append!(*ts, "{inner}}}\n");
            if when.arm.guard.is_none() {
                let next = emitter.name();
                append!(
                    *ts,
                    "{inner}type {next} = __VizePatterns.Subtract<{remaining}, {pattern_type}>;\n"
                );
                remaining = next;
            }
        }
    }
    if ctx.verification {
        let check = emitter.name();
        append!(*ts, "{inner}const ");
        let start = ts.len();
        ts.push_str(&check);
        map_range(
            mappings,
            start..ts.len(),
            (ctx.template_offset + subject.start) as usize
                ..(ctx.template_offset + subject.end) as usize,
        );
        append!(
            *ts,
            ": __VizePatterns.Exhaustiveness<{remaining}> = true;\n{inner}void {check};\n"
        );
    }
    append!(*ts, "{indent}}}\n");
    if guard.is_some() {
        append!(*ts, "{indent}}}\n");
    }
}

/// Subject, value-pattern and guard facts are emitted by the typed scope, not
/// again as generic custom-directive expressions in the enclosing scope.
pub(super) fn expression_ranges(summary: &Croquis) -> FxHashSet<(u32, u32)> {
    let mut ranges = FxHashSet::default();
    for scope in summary.scopes.iter() {
        match scope.data() {
            ScopeData::VMatch(subject) => {
                ranges.insert((subject.start, subject.end));
            }
            ScopeData::VWhen(when) => {
                value_ranges(&when.arm.pattern, when.offset, &mut ranges);
                if let Some(guard) = &when.arm.guard {
                    ranges.insert((when.offset + guard.span.start, when.offset + guard.span.end));
                }
            }
            _ => {}
        }
    }
    ranges
}

fn value_ranges(pattern: &MatchPattern, offset: u32, ranges: &mut FxHashSet<(u32, u32)>) {
    match &pattern.kind {
        PatternKind::Value(value) => {
            ranges.insert((offset + value.span.start, offset + value.span.end));
        }
        PatternKind::As { pattern, .. } => value_ranges(pattern, offset, ranges),
        PatternKind::Object { properties, .. } => {
            for property in properties {
                value_ranges(&property.pattern, offset, ranges);
            }
        }
        PatternKind::Array { elements, .. } | PatternKind::Or(elements) => {
            for element in elements {
                value_ranges(element, offset, ranges);
            }
        }
        _ => {}
    }
}
