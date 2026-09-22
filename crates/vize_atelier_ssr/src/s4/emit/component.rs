//! Components and the server-side built-ins: `ssrRenderComponent`,
//! `ssrRenderTeleport`, `ssrRenderSuspense`, and the transparent
//! `<Transition>` / `<KeepAlive>` wrappers.

use vize_atelier_core::RuntimeHelper;
use vize_atelier_core::codegen::document::EmitDocument;
use vize_s0::{String, ToCompactString, cstr};
use vize_s1_to_s2::{TransformContent, decode_template_entities};
use vize_s2::op::{self as s2, DynamicName};

use super::attrs::{Attached, admit_value};
use super::{Emitter, Flags, Result, plan_source, require_dynamic};
use crate::codegen::element::props::{
    component_prop_entry, component_props_object, quoted_js_string,
};
use crate::s4::string_plan::{
    SsrSegmentSource as Source, SsrStringPayloadKind, SsrStringSegment,
    SsrStringSegmentKind as Kind,
};
use crate::s4::{AdmissionFailure, LegacyReason};

/// A dynamic component's props leave out the static-name `is` spellings.
pub(super) fn without_is<'r, 'a>(
    attached: &Attached<'r, 'a>,
) -> std::vec::Vec<SsrStringSegment<'r, 'a>> {
    attached
        .iter()
        .copied()
        .filter(|segment| match segment.source {
            Source::Attribute(attr) => attr.name != "is",
            Source::Binding(s2::BindingOp::Bind(bind)) => {
                !matches!(bind.name, Some(DynamicName::Static("is")))
            }
            _ => true,
        })
        .collect()
}

/// Built-ins the server renders as their children.
fn is_transparent_builtin(name: &str) -> bool {
    matches!(
        name,
        "Transition"
            | "transition"
            | "BaseTransition"
            | "base-transition"
            | "KeepAlive"
            | "keep-alive"
    )
}

/// Component attachments the plan emitter owns. `v-slot` carriers and Vue 2
/// sugar keep the legacy lane; directives the legacy prop walk ignores stay
/// ignored.
fn admit(attached: &Attached<'_, '_>, owner_fact: u32) -> Result<()> {
    for segment in attached {
        match (segment.kind, segment.source) {
            (Kind::StaticAttribute, Source::Attribute(_)) if segment.fact == owner_fact => {}
            (_, Source::Binding(binding)) => {
                // Only the bindings that render an expression must read a
                // dynamic fact; ignored directives keep their own partition.
                if matches!(
                    binding,
                    s2::BindingOp::Bind(_)
                        | s2::BindingOp::On(_)
                        | s2::BindingOp::Model(_)
                        | s2::BindingOp::VueShow(_)
                ) {
                    require_dynamic(segment)?;
                }
                match binding {
                    s2::BindingOp::Bind(bind) => admit_value(bind.value.as_ref())?,
                    s2::BindingOp::On(on) => {
                        if let Some(handler) = &on.handler {
                            admit_value(Some(handler))?;
                        }
                    }
                    s2::BindingOp::Model(model) => admit_value(Some(&model.contract.read))?,
                    s2::BindingOp::VueShow(show) => admit_value(Some(&show.value))?,
                    s2::BindingOp::VueHtml(_)
                    | s2::BindingOp::VueText(_)
                    | s2::BindingOp::VueDirective(_)
                    | s2::BindingOp::VueOnce(_)
                    | s2::BindingOp::VueMemo(_)
                    | s2::BindingOp::VueCloak(_) => {}
                    s2::BindingOp::SlotContent(_) => {}
                    s2::BindingOp::VueSlotScope(_) => {
                        return Err(LegacyReason::Operation.into());
                    }
                    s2::BindingOp::VueSync(_) | s2::BindingOp::VueCssBind(_) => {
                        return Err(LegacyReason::Binding.into());
                    }
                }
            }
            _ => {
                return Err(AdmissionFailure::Invalid(
                    "string plan attached segment does not belong to its component",
                ));
            }
        }
    }
    Ok(())
}

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    pub(super) fn component(
        &mut self,
        open: SsrStringSegment<'r, 'a>,
        component: &'r s2::ComponentOp<'a>,
        inherit: bool,
    ) -> Result<()> {
        let name = plan_source(&open, SsrStringPayloadKind::ComponentName)?;
        self.pos += 1;
        let attached = self.take_attached(component.attributes.len() + component.bindings.len())?;
        let attached = attached.as_slice();
        admit(attached, open.fact)?;
        let no_inherit = Flags {
            as_fragment: false,
            disable_nested_fragments: false,
            inherit_attrs: false,
        };
        match name {
            "Suspense" | "suspense" => self.suspense(no_inherit)?,
            "Teleport" | "teleport" => self.teleport(attached, no_inherit)?,
            _ if is_transparent_builtin(name) => self.children(Flags {
                inherit_attrs: inherit,
                ..no_inherit
            })?,
            "component" | "Component" => self.dynamic_component(component, attached, inherit)?,
            _ => self.render_component(name, component, attached, inherit)?,
        }
        self.close(Kind::CloseComponent, |source| {
            matches!(source, Source::Component(closed) if core::ptr::eq(*closed, component))
        })
    }

    /// `_push(_ssrRenderComponent(callee, props, slots, _parent))`.
    fn render_component(
        &mut self,
        name: &str,
        component: &'r s2::ComponentOp<'a>,
        attached: &Attached<'_, '_>,
        inherit: bool,
    ) -> Result<()> {
        let content = self.pos;
        let slots = if component.children.ops.is_empty() {
            None
        } else {
            Some(self.component_slots(component, content)?)
        };
        let binding = self.ctx.resolve_component_binding_expr(name);
        let props = self.component_props(attached)?;
        let props = self.with_scope_id_prop(props);
        let props = self.with_fallthrough_attrs(props, inherit);
        self.ctx.flush_push();
        self.ctx.use_ssr_helper(RuntimeHelper::SsrRenderComponent);
        // The resolved name maps to the authored tag, as the AST walker's does.
        let callee = match binding {
            Some(binding) => EmitDocument::from(binding),
            None => self
                .ctx
                .component_callee(name, Some(component.span.start + 1)),
        };
        self.ctx.push_indent();
        self.ctx.push("_push(_ssrRenderComponent(");
        self.ctx.push_spanned(&callee);
        self.ctx.push(", ");
        self.ctx.push(&props);
        self.ctx.push(", ");
        match &slots {
            Some(slots) => self.emit_slots_object(slots)?,
            None => self.ctx.push("null"),
        }
        self.ctx.push(", _parent");
        if self.ctx.with_slot_scope_id {
            self.ctx.push(", _scopeId");
        }
        self.ctx.push("))\n");
        self.pos = self.region_end(content)?;
        Ok(())
    }

    /// `<component :is>`: `_ssrRenderVNode(_push, _createVNode(
    /// _resolveDynamicComponent(is), props, vnodeSlots), _parent)`. The
    /// legacy lane registers `ssrRenderComponent` before it branches.
    fn dynamic_component(
        &mut self,
        component: &'r s2::ComponentOp<'a>,
        attached: &Attached<'_, '_>,
        inherit: bool,
    ) -> Result<()> {
        let callee = self.dynamic_callee(attached)?;
        let props = self.component_props(&without_is(attached))?;
        let props = self.with_scope_id_prop(props);
        let props = self.with_fallthrough_attrs(props, inherit);
        let slots = if component.children.ops.is_empty() {
            "null".to_compact_string()
        } else {
            self.vnode_slots(component)?
        };
        self.ctx.flush_push();
        self.ctx.use_ssr_helper(RuntimeHelper::SsrRenderComponent);
        self.ctx.use_ssr_helper(RuntimeHelper::SsrRenderVNode);
        self.ctx.use_core_helper(RuntimeHelper::CreateVNode);
        self.ctx.push_indent();
        self.ctx.push("_ssrRenderVNode(_push, _createVNode(");
        self.ctx.push(&callee);
        self.ctx.push(", ");
        self.ctx.push(&props);
        self.ctx.push(", ");
        self.ctx.push(&slots);
        self.ctx.push("), _parent");
        if self.ctx.with_slot_scope_id {
            self.ctx.push(", _scopeId");
        }
        self.ctx.push(")\n");
        Ok(())
    }

    /// `_resolveDynamicComponent(is)`, with `null` when no `is` is spelled.
    pub(super) fn dynamic_callee(&mut self, attached: &Attached<'_, '_>) -> Result<String> {
        self.ctx
            .use_core_helper(RuntimeHelper::ResolveDynamicComponent);
        let target = self.static_or_bound(attached, "is")?;
        let target = target.as_deref().unwrap_or("null");
        Ok(cstr!("_resolveDynamicComponent({target})"))
    }

    fn with_scope_id_prop(&mut self, props: String) -> String {
        let options = self.ctx.options;
        let Some(scope_id) = options.scope_id.as_deref() else {
            return props;
        };
        let scope_props = component_props_object(&[component_prop_entry(scope_id, "\"\"", false)]);
        if props == "null" {
            return scope_props;
        }
        self.ctx.use_core_helper(RuntimeHelper::MergeProps);
        cstr!("_mergeProps({props}, {scope_props})")
    }

    fn with_fallthrough_attrs(&mut self, props: String, inherit: bool) -> String {
        if !inherit {
            return props;
        }
        if props == "null" || props == "_attrs" {
            return "_attrs".to_compact_string();
        }
        self.ctx.use_core_helper(RuntimeHelper::MergeProps);
        cstr!("_mergeProps({props}, _attrs)")
    }

    /// `_ssrRenderTeleport(_push, (_push) => { ... }, to, disabled, _parent)`.
    fn teleport(&mut self, attached: &Attached<'_, '_>, flags: Flags) -> Result<()> {
        let target = self.static_or_bound(attached, "to")?;
        let disabled = self.static_or_bound(attached, "disabled")?;
        self.ctx.flush_push();
        self.ctx.use_ssr_helper(RuntimeHelper::SsrRenderTeleport);
        self.ctx.push_indent();
        self.ctx.push("_ssrRenderTeleport(_push, (_push) => {\n");
        self.ctx.indent_level += 1;
        self.children(flags)?;
        self.ctx.flush_push();
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("}, ");
        self.ctx.push(target.as_deref().unwrap_or("undefined"));
        self.ctx.push(", ");
        self.ctx.push(disabled.as_deref().unwrap_or("false"));
        self.ctx.push(", _parent)\n");
        Ok(())
    }

    /// The first static attribute or static-name bind called `name`.
    fn static_or_bound(&self, attached: &Attached<'_, '_>, name: &str) -> Result<Option<String>> {
        for segment in attached {
            match segment.source {
                Source::Attribute(attr) if attr.name == name => {
                    return Ok(Some(match attr.value {
                        Some(value) => quoted_js_string(&decode_template_entities(value)),
                        None => "true".to_compact_string(),
                    }));
                }
                Source::Binding(s2::BindingOp::Bind(bind)) if matches!(bind.name, Some(DynamicName::Static(bound)) if bound == name) =>
                {
                    let value = bind.value.as_ref().ok_or(LegacyReason::Binding)?;
                    return self.expr(value, TransformContent::Decoded).map(Some);
                }
                _ => {}
            }
        }
        Ok(None)
    }

    /// `_ssrRenderSuspense(_push, { default: () => { ... }, _: 1 })`.
    fn suspense(&mut self, flags: Flags) -> Result<()> {
        self.ctx.flush_push();
        self.ctx.use_ssr_helper(RuntimeHelper::SsrRenderSuspense);
        self.ctx.push_indent();
        self.ctx.push("_ssrRenderSuspense(_push, {\n");
        self.ctx.indent_level += 1;
        self.ctx.push_indent();
        self.ctx.push("default: () => {\n");
        self.ctx.indent_level += 1;
        self.children(flags)?;
        self.ctx.flush_push();
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("},\n");
        self.ctx.push_indent();
        self.ctx.push("_: 1\n");
        self.ctx.indent_level -= 1;
        self.ctx.push_indent();
        self.ctx.push("})\n");
        Ok(())
    }
}
