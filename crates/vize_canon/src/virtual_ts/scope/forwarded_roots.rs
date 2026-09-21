//! The props a generic component forwards to its root, per instantiation.
//!
//! A generic component that falls its attributes through to a component root
//! forwards that root's props and listeners to its own parent. The root is
//! instantiated by the props the template binds on it, and those may mention
//! the component's own type parameters (`<basic :foo="bar" />` with
//! `bar?: T`), so the forwarded surface is only known inside the template,
//! where both the authored expressions and `T` are in scope. The Vue toolchain
//! reads it from the root's own call in the same place.
//!
//! The probes repeat the root's bound props to instantiate its resolvers. They
//! own no diagnostics: the root's mapped prop checks report every problem.

use vize_carton::{CompactString, FxHashSet, String, append, cstr};
use vize_croquis::{Croquis, analysis::ComponentUsage};

use crate::virtual_ts::{
    component_reference::resolved_component_binding_reference,
    expressions::{generated_prop_value, rewrite_reserved_template_binding},
    helpers::to_camel_case,
    template_binding_access::TemplateBindingAccess,
    types::VirtualTsOptions,
};

/// The value the template scope returns next to its inferred slots.
pub(crate) const FORWARDED_RETURN_KEY: &str = "__vizeForwarded";
/// The inferred slots, once the template scope returns more than them.
pub(crate) const SLOTS_RETURN_KEY: &str = "__vizeSlots";

/// Instantiates a root's props from the props bound on it. A root that has no
/// resolver is not generic, and its declared surface is already exact.
pub(crate) const FORWARDED_ROOT_HELPERS: &str = r#"type __VizeForwardedPropsFactory<C> = __VizeIsAny<C> extends true ? (component: C) => (props: any) => {} : C extends { __vizeResolveProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : (component: C) => (props: any) => __VizeComponentFallthroughProps<C> : (component: C) => (props: any) => __VizeComponentFallthroughProps<C>;
"#;

pub(super) struct ForwardedRootContext<'a> {
    pub(super) summary: &'a Croquis,
    pub(super) options: &'a VirtualTsOptions,
    pub(super) syntactic_type_only_imported_names: &'a FxHashSet<CompactString>,
    pub(super) template_binding_access: &'a TemplateBindingAccess,
}

/// Emit one probe pair per forwarded root and return the type of everything
/// they forward, or `None` when no root resolves to a component binding.
pub(super) fn emit_forwarded_root_probes(
    ts: &mut String,
    ctx: &ForwardedRootContext<'_>,
    root_starts: &[u32],
) -> Option<String> {
    let mut forwarded = String::default();
    for (index, usage) in ctx
        .summary
        .component_usages
        .iter()
        .filter(|usage| root_starts.contains(&usage.start))
        .enumerate()
    {
        let Some(component) = resolved_component_binding_reference(
            ctx.summary,
            ctx.options,
            ctx.syntactic_type_only_imported_names,
            usage.name.as_str(),
        ) else {
            continue;
        };
        if forwarded.is_empty() {
            ts.push_str(
                "\n  // Forwarded root props, instantiated by the props the template binds\n",
            );
        }
        for (kind, factory) in [
            ("props", "__VizeForwardedPropsFactory"),
            ("emits", "__VizeEmitPropsFactory"),
        ] {
            let name = cstr!("__vize_forwarded_{kind}_{index}");
            emit_probe(ts, ctx, usage, component.as_str(), factory, name.as_str());
            if !forwarded.is_empty() {
                forwarded.push_str(" & ");
            }
            append!(forwarded, "typeof {name}");
        }
    }
    (!forwarded.is_empty()).then_some(forwarded)
}

fn emit_probe(
    ts: &mut String,
    ctx: &ForwardedRootContext<'_>,
    usage: &ComponentUsage,
    component: &str,
    factory: &str,
    name: &str,
) {
    let guard = usage.vif_guard.as_ref().map(|guard| {
        rewrite_reserved_template_binding(guard.as_str(), ctx.template_binding_access)
            .unwrap_or_else(|| guard.clone())
    });
    let call = cstr!("(undefined as unknown as {factory}<typeof {component}>)({component})({{");
    let indent = if guard.is_some() { "    " } else { "  " };
    if let Some(guard) = guard.as_deref() {
        append!(*ts, "  const {name} = (() => {{\n");
        append!(
            *ts,
            "{indent}// @ts-ignore Inference-only guard and call; the root's mapped prop checks own diagnostics.\n"
        );
        append!(*ts, "{indent}if ({guard}) return {call}\n");
    } else {
        append!(
            *ts,
            "{indent}// @ts-ignore Inference-only call; the root's mapped prop checks own diagnostics.\n"
        );
        append!(*ts, "{indent}const {name} = {call}\n");
    }
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
    if guard.is_some() {
        ts.push_str("    return undefined as never;\n  })();\n");
    }
}
