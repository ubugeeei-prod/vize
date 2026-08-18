use std::ops::Range;

use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::croquis::{ComponentUsage, EventListener};
use vize_croquis::{Croquis, ScopeKind};

use crate::virtual_ts::{
    component_reference::component_binding_reference, expressions::rewrite_reserved_template_prop,
    helpers::to_camel_case, types::VizeMapping,
};

use super::component_navigation::{is_ts_identifier, push_ts_single_quoted_literal};
use super::context::{ComponentPropsContext, VForPropsContext};
use super::event_handler::event_name_source_range;
use crate::virtual_ts::expressions::generated_prop_value;

pub(super) fn emit_event_references(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &ComponentPropsContext<'_>,
    checkable_usages: &[(usize, &ComponentUsage)],
) {
    let navigation_ctx = EventNavigationContext {
        template_source: ctx.template_source,
        template_offset: ctx.template_offset,
        template_prop_names: ctx.template_prop_names,
        preserve_event_navigation: ctx.preserve_event_navigation,
    };
    for &(idx, usage) in checkable_usages {
        if is_closure_scoped(ctx.summary, usage) {
            continue;
        }
        let component_ref = component_binding_reference(
            ctx.summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        );
        emit_usage_event_references(
            ts,
            mappings,
            &navigation_ctx,
            idx,
            usage,
            component_ref.as_str(),
            "  ",
        );
    }
}

pub(super) fn emit_scoped_event_references(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &VForPropsContext<'_>,
    usages: &[(usize, &ComponentUsage)],
    indent: &str,
) {
    let navigation_ctx = EventNavigationContext {
        template_source: ctx.source_context.template,
        template_offset: ctx.source_context.offset,
        template_prop_names: ctx.template_prop_names,
        preserve_event_navigation: ctx.preserve_event_navigation,
    };
    for &(idx, usage) in usages {
        let component_ref = component_binding_reference(
            ctx.summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        );
        emit_usage_event_references(
            ts,
            mappings,
            &navigation_ctx,
            idx,
            usage,
            component_ref.as_str(),
            indent,
        );
    }
}

struct EventNavigationContext<'a> {
    template_source: Option<&'a str>,
    template_offset: u32,
    template_prop_names: &'a FxHashSet<String>,
    preserve_event_navigation: bool,
}

fn is_closure_scoped(summary: &Croquis, usage: &ComponentUsage) -> bool {
    summary
        .scopes
        .get_scope(usage.scope_id)
        .is_some_and(|scope| matches!(scope.kind, ScopeKind::VFor | ScopeKind::VSlot))
}

fn emit_usage_event_references(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    ctx: &EventNavigationContext<'_>,
    idx: usize,
    usage: &ComponentUsage,
    component_ref: &str,
    indent: &str,
) {
    let resolved_events = cstr!("__vize_events_resolved_{idx}");
    let direct_events_ref = cstr!("__vize_events_nav_{idx}");
    let kebab_events_ref = cstr!("__vize_kebab_events_nav_{idx}");
    let model_events_ref = cstr!("__vize_model_events_nav_{idx}");
    let model_completion_ref = cstr!("__vize_model_events_completion_{idx}");
    let mut emitted_direct_ref = false;
    let mut emitted_kebab_ref = false;
    let mut emitted_model_ref = false;
    let mut emitted_model_completion_ref = false;
    let mut emitted_resolved_events = false;
    let guard = usage.vif_guard.as_ref().map(|guard| {
        rewrite_reserved_template_prop(guard.as_str(), ctx.template_prop_names)
            .unwrap_or_else(|| guard.clone())
    });
    let guarded_indent = guard.as_ref().map(|_| cstr!("{indent}  "));
    let event_indent = guarded_indent.as_deref().unwrap_or(indent);
    let mut emitted_guard = false;
    for event in &usage.events {
        let camel_event_name = to_camel_case(event.name.as_str());
        let is_complete_kebab = camel_event_name != event.name && !event.name.ends_with('-');
        let Some(source_range) = event_navigation_source_range(ctx, event) else {
            continue;
        };
        if !emitted_guard {
            if let Some(guard) = guard.as_deref() {
                append!(
                    *ts,
                    "{indent}// @ts-ignore Navigation-only guard; authored binding checks own diagnostics.\n{indent}if ({guard}) {{\n"
                );
            }
            emitted_guard = true;
        }
        let is_model_event = ctx.preserve_event_navigation && event.name.starts_with("update:");
        if !emitted_resolved_events && ctx.preserve_event_navigation {
            emit_resolved_events(
                ts,
                ctx,
                usage,
                component_ref,
                idx,
                resolved_events.as_str(),
                event_indent,
            );
            emitted_resolved_events = true;
        }
        if is_model_event {
            if !emitted_model_completion_ref {
                append!(
                    *ts,
                    "{event_indent}const {model_completion_ref} = {resolved_events};\n"
                );
                emitted_model_completion_ref = true;
            }
            append!(*ts, "{event_indent}void {model_completion_ref}[");
            let generated = push_ts_single_quoted_literal(ts, event.name.as_str());
            ts.push_str("];\n");
            mappings.push(VizeMapping {
                gen_range: generated,
                src_range: source_range.clone(),
                sub_spans: Vec::new(),
            });
        }
        let (events_ref, emitted_ref) = if is_model_event {
            (&model_events_ref, &mut emitted_model_ref)
        } else if is_complete_kebab {
            (&kebab_events_ref, &mut emitted_kebab_ref)
        } else {
            (&direct_events_ref, &mut emitted_direct_ref)
        };
        if !*emitted_ref {
            if ctx.preserve_event_navigation {
                append!(
                    *ts,
                    "{event_indent}const {events_ref} = {resolved_events};\n"
                );
            } else {
                append!(
                    *ts,
                    "{event_indent}const {events_ref} = undefined as unknown as __VizeComponentEvents<typeof {component_ref}> & Record<string, unknown>;\n"
                );
            }
            *emitted_ref = true;
        }

        append!(*ts, "{event_indent}void {events_ref}");
        let event_gen_range = if is_complete_kebab {
            if is_ts_identifier(camel_event_name.as_str()) {
                ts.push('.');
                let start = ts.len();
                ts.push_str(camel_event_name.as_str());
                start..ts.len()
            } else {
                ts.push('[');
                let range = push_ts_single_quoted_literal(ts, camel_event_name.as_str());
                ts.push(']');
                range
            }
        } else if is_ts_identifier(event.name.as_str()) {
            ts.push('.');
            let start = ts.len();
            ts.push_str(event.name.as_str());
            start..ts.len()
        } else {
            ts.push('[');
            let range = push_ts_single_quoted_literal(ts, event.name.as_str());
            ts.push(']');
            range
        };
        ts.push_str(";\n");
        mappings.push(VizeMapping {
            gen_range: event_gen_range,
            src_range: source_range,
            sub_spans: Vec::new(),
        });
    }
    if emitted_guard && guard.is_some() {
        append!(*ts, "{indent}}}\n");
    }
}

fn emit_resolved_events(
    ts: &mut String,
    ctx: &EventNavigationContext<'_>,
    usage: &ComponentUsage,
    component_ref: &str,
    idx: usize,
    resolved_events: &str,
    indent: &str,
) {
    append!(
        *ts,
        "{indent}type __vize_events_resolver_{idx} = typeof {component_ref} extends {{ __vizeResolveEvents?: infer __F }} ? (__F extends (...args: any[]) => any ? __F : (props: any) => __VizeComponentEvents<typeof {component_ref}>) : (props: any) => __VizeComponentEvents<typeof {component_ref}>;\n{indent}// @ts-ignore Inference-only call; the authored prop checker owns diagnostics.\n{indent}const {resolved_events} = (undefined as unknown as __vize_events_resolver_{idx})({{\n"
    );
    for prop in &usage.props {
        if prop.name_is_dynamic || prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
            continue;
        }
        let Some(value) = generated_prop_value(prop, ctx.template_prop_names) else {
            continue;
        };
        let name = to_camel_case(prop.name.as_str());
        append!(*ts, "{indent}  \"{name}\": {value},\n");
    }
    append!(*ts, "{indent}}});\n");
}

fn event_navigation_source_range(
    ctx: &EventNavigationContext<'_>,
    event: &EventListener,
) -> Option<Range<usize>> {
    if event.name_is_dynamic || event.name.is_empty() {
        return None;
    }
    event_name_source_range(
        ctx.template_source,
        ctx.template_offset,
        event.start..event.end,
        event.name.as_str(),
    )
}
