use crate::ir::{ComponentKind, CreateComponentIRNode, IRSlot, OperationNode};
use vize_carton::{cstr, FxHashMap, String, ToCompactString};

use super::{
    super::context::GenerateContext, component_props::generate_component_props_str,
    component_slots::generate_slot_fn, insertion::emit_insertion_state,
};

pub(super) fn component_resolution_var(tag: &str) -> String {
    let mut ident = String::with_capacity(tag.len() + 11);
    ident.push_str("_component_");
    for ch in tag.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            ident.push(ch);
        } else {
            ident.push('_');
        }
    }
    ident
}

pub(super) fn emit_component_resolution(ctx: &mut GenerateContext, component_var: &str, tag: &str) {
    if let Some(binding_expr) = ctx.resolve_component_binding_expr(tag) {
        ctx.push_line(&cstr!("const {} = {}", component_var, binding_expr));
        return;
    }

    ctx.use_helper("resolveComponent");
    ctx.push_line(&cstr!(
        "const {} = _resolveComponent(\"{}\")",
        component_var,
        tag
    ));
}

/// Generate CreateComponent
pub(super) fn generate_create_component(
    ctx: &mut GenerateContext,
    component: &CreateComponentIRNode<'_>,
    element_template_map: &FxHashMap<usize, usize>,
) {
    let tag = &component.tag;
    let kind = component.kind;
    let use_with_vapor_ctx = kind == ComponentKind::Suspense || kind == ComponentKind::KeepAlive;

    // Track if this component was already resolved by a parent (Suspense/KeepAlive)
    let was_already_resolved = ctx.is_component_resolved(tag.as_str());

    // For Suspense/KeepAlive, resolve inner components FIRST (before the outer component)
    if use_with_vapor_ctx {
        for slot in component.slots.iter() {
            for op in slot.block.operation.iter() {
                if let OperationNode::CreateComponent(inner_comp) = op {
                    if (inner_comp.kind == ComponentKind::Regular
                        || inner_comp.kind == ComponentKind::Suspense)
                        && !ctx.is_component_resolved(inner_comp.tag.as_str())
                    {
                        emit_component_resolution(
                            ctx,
                            component_resolution_var(inner_comp.tag.as_str()).as_str(),
                            inner_comp.tag.as_str(),
                        );
                        ctx.mark_component_resolved(inner_comp.tag.as_str());
                    }
                }
            }
        }
    }

    // Determine component variable and creation function based on kind
    let (component_var, create_fn): (String, &str) = match kind {
        ComponentKind::Dynamic => {
            ctx.use_helper("createDynamicComponent");
            let is_arg = if let Some(ref is_exp) = component.is_expr {
                let resolved = ctx.resolve_expression(is_exp.content.as_str());
                cstr!("() => ({})", resolved)
            } else {
                "null".to_compact_string()
            };
            (is_arg, "createDynamicComponent")
        }
        ComponentKind::Teleport => {
            ctx.use_helper("VaporTeleport");
            ctx.use_helper("createComponent");
            ("_VaporTeleport".to_compact_string(), "createComponent")
        }
        ComponentKind::KeepAlive => {
            ctx.use_helper("VaporKeepAlive");
            ctx.use_helper("createComponent");
            ("_VaporKeepAlive".to_compact_string(), "createComponent")
        }
        ComponentKind::Suspense => {
            ctx.use_helper("createComponentWithFallback");
            let comp_var = component_resolution_var(tag.as_str());
            if !ctx.is_component_resolved(tag.as_str()) {
                emit_component_resolution(ctx, comp_var.as_str(), tag.as_str());
                ctx.mark_component_resolved(tag.as_str());
            }
            (comp_var, "createComponentWithFallback")
        }
        ComponentKind::Regular => {
            ctx.use_helper("createComponentWithFallback");
            let comp_var = component_resolution_var(tag.as_str());
            if !ctx.is_component_resolved(tag.as_str()) {
                emit_component_resolution(ctx, comp_var.as_str(), tag.as_str());
                ctx.mark_component_resolved(tag.as_str());
            }
            (comp_var, "createComponentWithFallback")
        }
    };

    let props = generate_component_props_str(ctx, component);
    let has_slots = !component.slots.is_empty();

    emit_insertion_state(ctx, component.parent, component.anchor);

    // Check if this is a simple inner component (pre-resolved, no props, no slots)
    // In that case, emit simplified call: _createComponentWithFallback(_component_Foo)
    let is_pre_resolved = was_already_resolved;
    if is_pre_resolved && !has_slots && props == "null" {
        ctx.push_line(&cstr!(
            "const n{} = _{}({})",
            component.id,
            create_fn,
            component_var
        ));
        return;
    }

    // Start component creation line
    ctx.push_indent();
    ctx.push(&cstr!(
        "const n{} = _{}({}, {}",
        component.id,
        create_fn,
        component_var,
        props
    ));

    if has_slots {
        ctx.push(", {\n");
        ctx.indent();

        let mut static_slots: std::vec::Vec<&IRSlot<'_>> = std::vec::Vec::new();
        let mut dynamic_slots: std::vec::Vec<&IRSlot<'_>> = std::vec::Vec::new();
        for slot in component.slots.iter() {
            if slot.name.is_static {
                static_slots.push(slot);
            } else {
                dynamic_slots.push(slot);
            }
        }

        for (i, slot) in static_slots.iter().enumerate() {
            ctx.push_indent();
            ctx.push(&cstr!("\"{}\":", slot.name.content));
            generate_slot_fn(ctx, slot, element_template_map, use_with_vapor_ctx);
            if i < static_slots.len() - 1 || !dynamic_slots.is_empty() {
                ctx.push(",");
            }
            ctx.push("\n");
        }

        if !dynamic_slots.is_empty() {
            ctx.push_line("$: [");
            ctx.indent();
            for (i, slot) in dynamic_slots.iter().enumerate() {
                ctx.push_indent();
                ctx.push("() => ({\n");
                ctx.indent();
                let name_resolved = ctx.resolve_expression(slot.name.content.as_str());
                ctx.push_line(&cstr!("name: {},", name_resolved));
                ctx.push_indent();
                ctx.push("fn:");
                generate_slot_fn(ctx, slot, element_template_map, false);
                ctx.push("\n");
                ctx.deindent();
                ctx.push_indent();
                ctx.push("})");
                if i < dynamic_slots.len() - 1 {
                    ctx.push(",");
                }
                ctx.push("\n");
                ctx.deindent();
            }
            ctx.deindent();
            ctx.push_line("]");
        }

        ctx.deindent();
        ctx.push_indent();
        ctx.push("}, true)\n");
    } else {
        ctx.push(", null, true)\n");
    }

    // v-show after component creation
    if let Some(ref v_show) = component.v_show {
        ctx.use_helper("applyVShow");
        let resolved = ctx.resolve_expression(v_show.content.as_str());
        ctx.push_line(&cstr!(
            "_applyVShow(n{}, () => ({}))",
            component.id,
            resolved
        ));
    }
}
