//! `<slot>` outlets: `_ssrRenderSlot(_ctx.$slots, name, props, fallback, ...)`.

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, camelize, cstr};
use vize_s1_to_s2::{TransformContent, decode_template_entities};
use vize_s2::op::{self as s2, DynamicName};

use super::attrs::{Attached, admit_value};
use super::{Emitter, Flags, Result, plan_source, require_dynamic};
use crate::codegen::element::props::{
    component_prop_entry, component_props_object, normalize_prop_entries, quoted_js_string,
};
use crate::s4::string_plan::{
    SsrSegmentSource as Source, SsrStringPayloadKind, SsrStringSegment,
    SsrStringSegmentKind as Kind,
};
use crate::s4::{AdmissionFailure, LegacyReason};

/// Outlet attachments the plan emitter owns: static props and `v-bind`
/// (bindings other than `v-bind` render nothing on an outlet).
fn admit(attached: &Attached<'_, '_>, owner_fact: u32) -> Result<()> {
    for segment in attached {
        match (segment.kind, segment.source) {
            (Kind::StaticAttribute, Source::Attribute(_)) if segment.fact == owner_fact => {}
            (_, Source::Binding(binding)) => {
                if matches!(binding, s2::BindingOp::Bind(_) | s2::BindingOp::On(_)) {
                    require_dynamic(segment)?;
                }
                match binding {
                    // A second `name` source: the legacy lookup skips a
                    // value-less `name` and reads the later `:name`, while
                    // S2 settles on the first spelling.
                    s2::BindingOp::Bind(bind)
                        if matches!(bind.name, Some(DynamicName::Static("name"))) =>
                    {
                        return Err(LegacyReason::Binding.into());
                    }
                    s2::BindingOp::Bind(bind) => {
                        if let Some(DynamicName::Dynamic(argument)) = &bind.name {
                            super::attrs::admit_dynamic_key(argument)?;
                        }
                        admit_value(bind.value.as_ref())?;
                    }
                    s2::BindingOp::On(_) => {}
                    _ => return Err(LegacyReason::Binding.into()),
                }
            }
            _ => {
                return Err(AdmissionFailure::Invalid(
                    "string plan attached segment does not belong to its outlet",
                ));
            }
        }
    }
    Ok(())
}

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    pub(super) fn slot_outlet(
        &mut self,
        open: SsrStringSegment<'r, 'a>,
        slot: &'r s2::SlotOp<'a>,
    ) -> Result<()> {
        plan_source(&open, SsrStringPayloadKind::SlotName)?;
        self.pos += 1;
        let attached = self.take_attached(slot.attributes.len() + slot.bindings.len())?;
        let attached = attached.as_slice();
        admit(attached, open.fact)?;

        let name = self.outlet_name(slot)?;
        let props = self.slot_props(attached)?;
        self.ctx.flush_push();
        self.ctx.use_ssr_helper(RuntimeHelper::SsrRenderSlot);
        self.ctx.push_indent();
        self.ctx.push("_ssrRenderSlot(_ctx.$slots, ");
        self.ctx.push(&name);
        self.ctx.push(", ");
        self.ctx.push(&props);
        self.ctx.push(", ");
        if slot.fallback.ops.is_empty() {
            self.ctx.push("null");
        } else {
            self.ctx.push("() => {\n");
            self.ctx.indent_level += 1;
            let saved = core::mem::take(&mut self.ctx.current_template_parts);
            let fallback = self.children(Flags {
                as_fragment: false,
                disable_nested_fragments: false,
                inherit_attrs: false,
            });
            self.ctx.flush_push();
            self.ctx.current_template_parts = saved;
            fallback?;
            self.ctx.indent_level -= 1;
            self.ctx.push_indent();
            self.ctx.push("}");
        }
        self.ctx.push(", _push, _parent");
        if self.ctx.with_slot_scope_id || self.ctx.options.scope_id.is_some() {
            self.ctx.push(", _scopeId");
        }
        self.ctx.push(")\n");
        self.close(
            Kind::CloseSlot,
            |source| matches!(source, Source::Slot(closed) if core::ptr::eq(*closed, slot)),
        )
    }

    /// The outlet name: a quoted static name or the rewritten `:name`.
    pub(super) fn outlet_name(&self, slot: &s2::SlotOp<'_>) -> Result<String> {
        match &slot.name {
            DynamicName::Static(name) => Ok(quoted_js_string(&decode_template_entities(name))),
            DynamicName::Dynamic(expr) => self.expr(expr, TransformContent::Decoded),
        }
    }

    /// The outlet's slot props: camelized static props and bound props,
    /// merged after any object spreads.
    pub(super) fn slot_props(&mut self, attached: &Attached<'_, '_>) -> Result<String> {
        let mut entries = std::vec::Vec::new();
        let mut spreads = std::vec::Vec::new();
        for segment in attached {
            match segment.source {
                Source::Attribute(attr) => {
                    let name = plan_source(segment, SsrStringPayloadKind::AttributeName)?;
                    let value = attr
                        .value
                        .map(|value| quoted_js_string(&decode_template_entities(value)))
                        .unwrap_or_else(|| "true".to_compact_string());
                    entries.push(component_prop_entry(&camelize(name), &value, false));
                }
                Source::Binding(s2::BindingOp::Bind(bind)) => {
                    let value = bind.value.as_ref().ok_or(LegacyReason::Binding)?;
                    let value = self.expr(value, TransformContent::Decoded)?;
                    match &bind.name {
                        None => spreads.push(value),
                        Some(DynamicName::Static(name)) => {
                            let key = slot_prop_key(name, &bind.modifiers);
                            entries.push(component_prop_entry(&key, &value, false));
                        }
                        Some(name @ DynamicName::Dynamic(_)) => {
                            let key = self.dynamic_key(name)?;
                            entries.push(component_prop_entry(&key, &value, true));
                        }
                    }
                }
                _ => {}
            }
        }
        let entries = normalize_prop_entries(entries);
        let object = if entries.is_empty() {
            "{}".to_compact_string()
        } else {
            component_props_object(&entries)
        };
        if spreads.is_empty() {
            return Ok(object);
        }
        self.ctx.use_core_helper(RuntimeHelper::MergeProps);
        if !entries.is_empty() {
            spreads.push(object);
        }
        Ok(cstr!("_mergeProps({})", spreads.join(", ")))
    }
}

/// Outlet prop keys are camelized; `.prop` / `.attr` still prefix.
fn slot_prop_key(name: &str, modifiers: &[&str]) -> String {
    let base = camelize(name);
    if modifiers.contains(&"prop") {
        return cstr!(".{base}");
    }
    if modifiers.contains(&"attr") {
        return cstr!("^{base}");
    }
    base
}
