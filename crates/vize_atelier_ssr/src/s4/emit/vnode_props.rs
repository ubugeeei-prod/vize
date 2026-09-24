//! VNode fallback props and slot outlets.

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, camelize, cstr};
use vize_s1_to_s2::{TransformContent, decode_template_entities};
use vize_s2::op::{self as s2, DynamicName};

use super::attrs::Attached;
use super::{Emitter, Result, plan_source};
use crate::codegen::element::props::{
    component_prop_entry, component_props_object, normalize_prop_entries, quoted_js_string,
    wrap_call,
};
use crate::s4::LegacyReason;
use crate::s4::string_plan::{
    SsrSegmentSource as Source, SsrStringPayloadKind, SsrStringSegmentKind as Kind,
};

/// `v-bind` key modifiers: `.camel` camelizes, `.prop` / `.attr` prefix.
fn bound_key(name: &str, modifiers: &[&str]) -> String {
    if modifiers.contains(&"camel") {
        return camelize(name);
    }
    if modifiers.contains(&"prop") {
        return cstr!(".{name}");
    }
    if modifiers.contains(&"attr") {
        return cstr!("^{name}");
    }
    name.to_compact_string()
}

impl<'r, 'a> Emitter<'_, 'r, 'a, '_, '_, '_> {
    /// `build_plain_vnode_props_with_key`: static attributes and `v-bind`
    /// only (other directives render nothing in the fallback), the optional
    /// loop key first and the scoped-style id last.
    pub(super) fn vnode_element_props(
        &mut self,
        attached: &Attached<'_, '_>,
        key: Option<&str>,
    ) -> Result<String> {
        let options = self.ctx.options;
        let scope_id = options.scope_id.as_deref();
        if attached.is_empty() && key.is_none() {
            return Ok(match scope_id {
                Some(scope_id) => {
                    component_props_object(&[component_prop_entry(scope_id, "\"\"", false)])
                }
                None => "null".to_compact_string(),
            });
        }
        let mut entries = std::vec::Vec::new();
        let mut spreads = std::vec::Vec::new();
        if let Some(key) = key {
            entries.push(component_prop_entry("key", key, false));
        }
        for segment in attached {
            match segment.source {
                Source::Attribute(attr) => {
                    let name = plan_source(segment, SsrStringPayloadKind::AttributeName)?;
                    let value = attr
                        .value
                        .map(|value| quoted_js_string(&decode_template_entities(value)))
                        .unwrap_or_else(|| "\"\"".to_compact_string());
                    entries.push(component_prop_entry(name, &value, false));
                }
                Source::Binding(s2::BindingOp::Bind(bind)) => {
                    let value = match &bind.value {
                        Some(value) => self.expr(value, TransformContent::Decoded)?,
                        None => return Err(LegacyReason::Binding.into()),
                    };
                    match bind.name {
                        None => spreads.push(value),
                        Some(DynamicName::Static(name)) => {
                            let key = bound_key(name, &bind.modifiers);
                            entries.push(component_prop_entry(&key, &value, false));
                        }
                        Some(DynamicName::Dynamic(_)) => {
                            return Err(LegacyReason::Binding.into());
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(scope_id) = scope_id {
            entries.push(component_prop_entry(scope_id, "\"\"", false));
        }
        let entries = normalize_prop_entries(entries);
        if spreads.is_empty() {
            return Ok(if entries.is_empty() {
                "null".to_compact_string()
            } else {
                component_props_object(&entries)
            });
        }
        self.ctx.use_core_helper(RuntimeHelper::MergeProps);
        if !entries.is_empty() {
            spreads.push(component_props_object(&entries));
        }
        Ok(wrap_call("_mergeProps", &spreads.join(", ")))
    }

    /// `_renderSlot(_ctx.$slots, name, props[, () => [fallback]])`.
    pub(super) fn vnode_outlet(&mut self, slot: &'r s2::SlotOp<'a>) -> Result<String> {
        let open = self.segment(self.pos)?;
        plan_source(&open, SsrStringPayloadKind::SlotName)?;
        self.pos += 1;
        let attached = self.take_attached(slot.attributes.len() + slot.bindings.len())?;
        let attached = attached.as_slice();
        self.ctx.use_core_helper(RuntimeHelper::RenderSlot);
        let name = self.outlet_name(slot)?;
        let props = self.slot_props(attached)?;
        let mut out = cstr!("_renderSlot(_ctx.$slots, {name}, {props}");
        if !slot.fallback.ops.is_empty() {
            let fallback = super::vnode::array(&self.vnode_region()?);
            out.push_str(", () => ");
            out.push_str(&fallback);
        }
        out.push(')');
        self.close(
            Kind::CloseSlot,
            |source| matches!(source, Source::Slot(closed) if core::ptr::eq(*closed, slot)),
        )?;
        Ok(out)
    }
}
