//! The instance a component ref holds, as the template instantiates it.
//!
//! `<Generic ref="item" :foo="1" />` registers the instance of `Generic<1>`,
//! not of `Generic<unknown>`: the props bound next to the ref choose the type
//! parameters. Like a forwarded root, that is only known where the authored
//! expressions are in scope, so the template calls the component with them and
//! returns the result to the ref registry of the setup scope.
//!
//! A ref inside `v-for` is instantiated inside a replay of its loops. The
//! probe is a `var`, which a block does not confine, so the template scope can
//! still name it. A slot scope is a callback, and a ref inside one keeps the
//! declared instance type.

use vize_carton::{CompactString, FxHashMap, FxHashSet, String, append, cstr};
use vize_croquis::{Croquis, Scope, ScopeKind, analysis::ComponentUsage};
use vize_l0::ts;

use crate::virtual_ts::{
    component_reference::resolved_component_binding_reference,
    expressions::{generated_prop_value, rewrite_reserved_template_binding},
    helpers::to_camel_case,
    template_binding_access::TemplateBindingAccess,
    types::VirtualTsOptions,
};

use super::emit::emit_v_for_loop_open;

/// The ref instances the template scope returns, keyed by usage start.
pub(crate) const REFS_RETURN_KEY: &str = "__vizeRefs";

/// A component that is not callable has no type parameters to choose, and its
/// declared instance is already exact. The registry decides that from the
/// component alone and only then reads the probe: the probe depends on the
/// props bound next to the ref, and `<Chart ref="chart" :config="config" />`
/// with a `config` computed from `chart.value` would otherwise make `chart`
/// depend on itself for every component, not only for the generic ones whose
/// instance really is a function of their props.
macro_rules! template_ref_instance_helper {
    () => {
        "  type $TemplateRefInstance<_T, _K extends keyof _T, _C> = 0 extends (1 & _C) ? $TemplateComponentRef<_C> : _C extends (props: any, ctx?: any, expose?: any, ...args: any[]) => any ? _T[_K] extends () => infer _R ? _R extends { __ctx?: infer _X } ? NonNullable<_X> extends { expose: (exposed: infer _E) => any } ? NonNullable<_E> : $TemplateComponentRef<_C> : $TemplateComponentRef<_C> : $TemplateComponentRef<_C> : $TemplateComponentRef<_C>;\n"
    };
}

pub(crate) const REF_INSTANCE_HELPERS: &str = concat!(
    "  ",
    include_str!("../helpers/declarations/ref-instance-factory.d.ts.txt"),
    template_ref_instance_helper!()
);

// Only the factory is provider-independent: every non-keyword reference is
// its own type parameter or an inferred local. Keep the DOM/instance helper
// in setup scope, where authored provider and local type bindings still apply.
pub(crate) const REF_INSTANCE_HELPERS_HOISTED: &str = concat!(
    "  type $RefInstanceFactory<_C> = import(\"vize:virtual-ts\").$RefInstanceFactory<_C>;\n",
    template_ref_instance_helper!()
);

/// Template-relative starts of the component usages whose ref is instantiated:
/// a static `ref` next to at least one bound prop, outside every slot scope.
pub(crate) fn instantiated_ref_starts(summary: &Croquis) -> Vec<u32> {
    vize_croquis::facts::component_usage_list(summary)
        .iter()
        .filter(|usage| {
            let mut has_ref = false;
            let mut binds_props = false;
            for prop in &usage.props {
                match prop.name.as_str() {
                    _ if prop.name_is_dynamic => {}
                    "ref" => has_ref = !prop.is_dynamic && prop.value.is_some(),
                    "key" => {}
                    _ => binds_props |= prop.value.is_some(),
                }
            }
            has_ref && binds_props && enclosing_loops(summary, usage).is_some()
        })
        .map(|usage| usage.start)
        .collect()
}

/// The `v-for` scopes around a usage, outermost first, or `None` when a scope
/// that is not a block encloses it.
fn enclosing_loops<'a>(summary: &'a Croquis, usage: &ComponentUsage) -> Option<Vec<&'a Scope>> {
    let mut loops = Vec::new();
    let mut cursor = summary.scopes.get_scope(usage.scope_id);
    while let Some(scope) = cursor {
        match scope.kind {
            ScopeKind::VFor => loops.push(scope),
            ScopeKind::VSlot | ScopeKind::VMatch | ScopeKind::VWhen => return None,
            _ => {}
        }
        cursor = scope.parent().and_then(|id| summary.scopes.get_scope(id));
    }
    loops.reverse();
    Some(loops)
}

pub(super) struct RefInstanceContext<'a> {
    pub(super) summary: &'a Croquis,
    pub(super) options: &'a VirtualTsOptions,
    pub(super) syntactic_type_only_imported_names: &'a FxHashSet<CompactString>,
    pub(super) template_binding_access: &'a TemplateBindingAccess<'a>,
    pub(super) template_offset: u32,
    /// The `v-if` guards that narrow a `v-for` source from outside its loop.
    pub(super) vfor_enclosing_guards: &'a FxHashMap<u32, String>,
}

/// Emit one probe per instantiated ref and return the record the template
/// scope returns for them. Every requested start is a key of it, so the
/// registry can index it without knowing which probes were possible.
///
/// Each entry is a thunk: a function's return type is only resolved when it is
/// asked for, so reading one ref from the registry does not resolve the props
/// of another. `<Generic ref="b" :foo="a.foo" />` would otherwise make the
/// type of `a` depend on itself through the record it is read from.
pub(super) fn emit_ref_instance_probes(
    ts: &mut String,
    ctx: &RefInstanceContext<'_>,
    starts: &[u32],
) -> String {
    let mut record = String::default();
    ts! { &mut record =>
        {
            #(" ")
            #{
                for usage in vize_croquis::facts::component_usage_list(ctx.summary) {
                    if !starts.contains(&usage.start) {
                        continue;
                    }
                    let probe = resolved_component_binding_reference(
                        ctx.summary,
                        ctx.options,
                        ctx.syntactic_type_only_imported_names,
                        usage.name.as_str(),
                    )
                    .zip(enclosing_loops(ctx.summary, &usage))
                    .map(|(component, loops)| {
                        emit_probe(ts, ctx, &usage, component.as_str(), &loops)
                    });
                    match probe {
                        Some(name) => append!(*__ts, "\"{}\": () => {name}, ", usage.start),
                        None => append!(*__ts, "\"{}\": () => undefined, ", usage.start),
                    }
                }
            }
        }
    }
    record
}

fn emit_probe(
    ts: &mut String,
    ctx: &RefInstanceContext<'_>,
    usage: &ComponentUsage,
    component: &str,
    loops: &[&Scope],
) -> String {
    let name = cstr!("__vize_ref_{}", usage.start);
    append!(*ts, "\n  // Ref instance of <{}>\n", usage.name);
    let mut indent = String::from("  ");
    emit_ref_loops(
        ts,
        ctx,
        usage,
        component,
        loops,
        0,
        &mut indent,
        name.as_str(),
    );
    name
}

/// Replay enclosing loops from the outside in. Each loop's braces are closed
/// by [`emit_v_for_loop_open`], so the body of one loop is the next loop.
#[expect(
    clippy::too_many_arguments,
    reason = "one probe threaded through its loops"
)]
fn emit_ref_loops(
    ts: &mut String,
    ctx: &RefInstanceContext<'_>,
    usage: &ComponentUsage,
    component: &str,
    loops: &[&Scope],
    index: usize,
    indent: &mut String,
    name: &str,
) {
    if index == loops.len() {
        if let Some(raw) = usage.vif_guard.as_ref() {
            let guard =
                rewrite_reserved_template_binding(raw.as_str(), ctx.template_binding_access)
                    .unwrap_or_else(|| raw.clone());
            with_guard(ts, indent, guard.as_str(), |ts, indent| {
                emit_ref_call(ts, ctx, usage, component, indent.as_str(), name);
            });
        } else {
            emit_ref_call(ts, ctx, usage, component, indent.as_str(), name);
        }
        return;
    }
    let Some(&scope) = loops.get(index) else {
        return;
    };
    if let Some(guard) = ctx.vfor_enclosing_guards.get(&scope.id.as_u32()) {
        with_guard(ts, indent, guard.as_str(), |ts, indent| {
            emit_ref_loop(ts, ctx, usage, component, loops, scope, index, indent, name);
        });
    } else {
        emit_ref_loop(ts, ctx, usage, component, loops, scope, index, indent, name);
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "one probe threaded through its loops"
)]
fn emit_ref_loop(
    ts: &mut String,
    ctx: &RefInstanceContext<'_>,
    usage: &ComponentUsage,
    component: &str,
    loops: &[&Scope],
    scope: &Scope,
    index: usize,
    indent: &mut String,
    name: &str,
) {
    // The replayed loops own no diagnostics: the scopes themselves report.
    let mut unmapped = Vec::new();
    let at = indent.clone();
    emit_v_for_loop_open(
        ts,
        &mut unmapped,
        ctx.template_offset,
        None,
        at.as_str(),
        scope,
        ctx.template_binding_access,
        false,
        |ts, _mappings| {
            let mut nested = at.clone();
            nested.push_str("  ");
            emit_ref_loops(
                ts,
                ctx,
                usage,
                component,
                loops,
                index + 1,
                &mut nested,
                name,
            );
        },
    );
}

fn emit_ref_call(
    ts: &mut String,
    ctx: &RefInstanceContext<'_>,
    usage: &ComponentUsage,
    component: &str,
    indent: &str,
    name: &str,
) {
    append!(
        *ts,
        "{indent}// @ts-ignore Inference-only call; the usage's mapped prop checks own diagnostics.\n"
    );
    append!(
        *ts,
        "{indent}var {name} = (undefined as unknown as $RefInstanceFactory<typeof {component}>)({component})({{\n"
    );
    let mut emitted = FxHashSet::default();
    for prop in &usage.props {
        if prop.name_is_dynamic || matches!(prop.name.as_str(), "key" | "ref") {
            continue;
        }
        let Some(value) = generated_prop_value(prop, ctx.template_binding_access) else {
            continue;
        };
        let prop_name = to_camel_case(prop.name.as_str());
        if !emitted.insert(prop_name.clone()) {
            continue;
        }
        if super::is_inline_callback_prop(prop) {
            append!(
                *ts,
                "{indent}  // @ts-ignore Inference-only callback prop; the mapped prop owner checks diagnostics.\n"
            );
        }
        append!(*ts, "{indent}  \"{}\": {},\n", prop_name, value.as_str());
    }
    append!(*ts, "{indent}}});\n");
}

fn with_guard(
    ts: &mut String,
    indent: &mut String,
    guard: &str,
    body: impl FnOnce(&mut String, &mut String),
) {
    let at = indent.clone();
    let isolated = crate::virtual_ts::expressions::isolate_incomplete_expression(guard);
    let guard = isolated.as_str();
    append!(
        *ts,
        "{at}// @ts-ignore Inference-only guard; the authored binding checks own diagnostics.\n"
    );
    append!(*ts, "{at}if ({guard}) {{\n");
    indent.push_str("  ");
    body(ts, indent);
    append!(*ts, "{at}}}\n");
}

#[cfg(test)]
mod tests {
    use super::{REF_INSTANCE_HELPERS, instantiated_ref_starts};
    use vize_carton::Allocator;
    use vize_croquis::{Analyzer, AnalyzerOptions};

    fn starts(template: &str) -> Vec<u32> {
        let allocator = Allocator::new();
        let (root, _) = vize_armature::parse(&allocator, template);
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(
            "import Item from './Item.vue'\nimport List from './List.vue'\nconst n = 1",
        );
        analyzer.analyze_template(&root);
        instantiated_ref_starts(&analyzer.finish())
    }

    #[test]
    fn a_static_ref_next_to_bound_props_is_instantiated() {
        assert_eq!(starts(r#"<Item ref="item" :foo="n" />"#), [0]);
        assert_eq!(starts(r#"<Item ref="item" foo="text" />"#), [0]);
        assert_eq!(
            starts(r#"<div><Item v-for="i in 3" ref="items" :key="i" :foo="i" /></div>"#),
            [5]
        );
    }

    #[test]
    fn every_other_ref_keeps_its_declared_instance() {
        // Nothing to instantiate with, a ref the template computes, no ref at
        // all, and a usage inside a slot scope, which is a callback.
        for template in [
            r#"<Item ref="item" />"#,
            r#"<Item ref="item" :key="n" />"#,
            r#"<Item :ref="n" :foo="n" />"#,
            r#"<Item :foo="n" />"#,
            r#"<List v-slot="{ row }"><Item ref="item" :foo="row" /></List>"#,
        ] {
            assert_eq!(starts(template), Vec::<u32>::new(), "{template}");
        }
    }

    #[test]
    fn generated_generic_refs_use_the_open_inference_resolver() {
        assert_eq!(
            REF_INSTANCE_HELPERS
                .matches("__vizeResolveRef?: infer _F")
                .count(),
            1,
            "{REF_INSTANCE_HELPERS}",
        );
    }
}
