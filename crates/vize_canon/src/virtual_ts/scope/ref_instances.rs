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
pub(crate) const REF_INSTANCE_HELPERS: &str = "  type __VizeRefInstanceFactory<_C> = 0 extends (1 & _C) ? (component: _C) => (props: any) => undefined : _C extends (props: any, ctx?: any, expose?: any, ...args: any[]) => any ? (component: _C) => _C : (component: _C) => (props: any) => undefined;\n  type __VizeTemplateRefInstance<_T, _K extends keyof _T, _C> = 0 extends (1 & _C) ? __VizeTemplateComponentRef<_C> : _C extends (props: any, ctx?: any, expose?: any, ...args: any[]) => any ? _T[_K] extends () => infer _R ? _R extends { __ctx?: infer _X } ? NonNullable<_X> extends { expose: (exposed: infer _E) => any } ? NonNullable<_E> : __VizeTemplateComponentRef<_C> : __VizeTemplateComponentRef<_C> : __VizeTemplateComponentRef<_C> : __VizeTemplateComponentRef<_C>;\n";

/// Template-relative starts of the component usages whose ref is instantiated:
/// a static `ref` next to at least one bound prop, outside every slot scope.
pub(crate) fn instantiated_ref_starts(summary: &Croquis) -> Vec<u32> {
    summary
        .component_usages
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
    pub(super) template_binding_access: &'a TemplateBindingAccess,
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
    let mut record = String::from("{ ");
    for usage in &ctx.summary.component_usages {
        if !starts.contains(&usage.start) {
            continue;
        }
        let probe = resolved_component_binding_reference(
            ctx.summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        )
        .zip(enclosing_loops(ctx.summary, usage))
        .map(|(component, loops)| emit_probe(ts, ctx, usage, component.as_str(), &loops));
        match probe {
            Some(name) => append!(record, "\"{}\": () => {name}, ", usage.start),
            None => append!(record, "\"{}\": () => undefined, ", usage.start),
        }
    }
    record.push('}');
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
    let mut closers: Vec<String> = Vec::new();
    // The replayed loops own no diagnostics: the scopes themselves report.
    let mut unmapped = Vec::new();
    for scope in loops {
        if let Some(guard) = ctx.vfor_enclosing_guards.get(&scope.id.as_u32()) {
            open_guard(ts, &mut indent, &mut closers, guard.as_str());
        }
        emit_v_for_loop_open(
            ts,
            &mut unmapped,
            ctx.template_offset,
            None,
            indent.as_str(),
            scope,
            ctx.template_binding_access,
            false,
        );
        closers.push(cstr!("{indent}}}\n{indent}}}\n"));
        indent.push_str("  ");
    }
    if let Some(guard) = usage.vif_guard.as_ref() {
        let guard = rewrite_reserved_template_binding(guard.as_str(), ctx.template_binding_access)
            .unwrap_or_else(|| guard.clone());
        open_guard(ts, &mut indent, &mut closers, guard.as_str());
    }
    append!(
        *ts,
        "{indent}// @ts-ignore Inference-only call; the usage's mapped prop checks own diagnostics.\n"
    );
    append!(
        *ts,
        "{indent}var {name} = (undefined as unknown as __VizeRefInstanceFactory<typeof {component}>)({component})({{\n"
    );
    for prop in &usage.props {
        if prop.name_is_dynamic || matches!(prop.name.as_str(), "key" | "ref") {
            continue;
        }
        let Some(value) = generated_prop_value(prop, ctx.template_binding_access) else {
            continue;
        };
        if super::is_inline_callback_prop(prop) {
            append!(
                *ts,
                "{indent}  // @ts-ignore Inference-only callback prop; the mapped prop owner checks diagnostics.\n"
            );
        }
        append!(
            *ts,
            "{indent}  \"{}\": {},\n",
            to_camel_case(prop.name.as_str()),
            value.as_str()
        );
    }
    append!(*ts, "{indent}}});\n");
    for closer in closers.iter().rev() {
        ts.push_str(closer.as_str());
    }
    name
}

fn open_guard(ts: &mut String, indent: &mut String, closers: &mut Vec<String>, guard: &str) {
    append!(
        *ts,
        "{indent}// @ts-ignore Inference-only guard; the authored binding checks own diagnostics.\n"
    );
    append!(*ts, "{indent}if ({guard}) {{\n");
    closers.push(cstr!("{indent}}}\n"));
    indent.push_str("  ");
}

#[cfg(test)]
mod tests {
    use super::instantiated_ref_starts;
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
}
