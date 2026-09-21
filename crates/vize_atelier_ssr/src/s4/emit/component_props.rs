//! Component prop bags: source-ordered segments that never cross a spread or
//! dynamic-key boundary (the legacy `build_component_props` shape).

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, camelize, cstr};
use vize_s1_to_s2::{TransformContent, TransformExpressions, decode_template_entities};
use vize_s2::expr::ExprRef;
use vize_s2::op::{self as s2, DynamicName};

use super::attrs::Attached;
use super::{Emitter, Result, plan_source};
use crate::codegen::element::VNodePropEntry;
use crate::codegen::element::props::{
    component_prop_entry, component_props_object, is_valid_js_identifier, normalize_prop_entries,
    quoted_js_string, wrap_call,
};
use crate::s4::LegacyReason;
use crate::s4::string_plan::{SsrSegmentSource as Source, SsrStringPayloadKind};

enum Segment {
    Entries(std::vec::Vec<VNodePropEntry>),
    Spread(String),
}

/// A dynamic directive argument the plan emitter owns: a bare identifier,
/// which the legacy walker spells `_ctx.<name>` without any rewrite.
fn dynamic_key(name: &DynamicName<'_>) -> Result<Option<String>> {
    let DynamicName::Dynamic(expr) = name else {
        return Ok(None);
    };
    let source = expr.source();
    let simple = is_valid_js_identifier(source)
        && !matches!(source, "true" | "false" | "null" | "undefined");
    if !simple {
        return Err(LegacyReason::Binding.into());
    }
    Ok(Some(cstr!("_ctx.{source}")))
}

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

impl Emitter<'_, '_, '_, '_, '_, '_> {
    /// The component's prop bag, or `null` when it has no prop segments.
    pub(super) fn component_props(&mut self, attached: &Attached<'_, '_>) -> Result<String> {
        let mut segments = std::vec::Vec::new();
        let mut entries = std::vec::Vec::new();
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
                Source::Binding(binding) => {
                    if is_boundary(binding) {
                        flush(&mut segments, &mut entries);
                    }
                    match self.component_binding(binding, &mut entries)? {
                        Some(spread) => segments.push(Segment::Spread(spread)),
                        None if is_boundary(binding) => flush(&mut segments, &mut entries),
                        None => {}
                    }
                }
                _ => {}
            }
        }
        flush(&mut segments, &mut entries);
        let rendered: std::vec::Vec<String> = segments
            .into_iter()
            .map(|segment| self.segment_expression(segment))
            .collect();
        Ok(match rendered.as_slice() {
            [] => "null".to_compact_string(),
            [only] => only.clone(),
            _ => {
                self.ctx.use_core_helper(RuntimeHelper::MergeProps);
                cstr!("_mergeProps({})", rendered.join(", "))
            }
        })
    }

    fn segment_expression(&mut self, segment: Segment) -> String {
        match segment {
            Segment::Entries(entries) => {
                let dynamic = entries.iter().any(|entry| entry.dynamic());
                let object = component_props_object(&normalize_prop_entries(entries));
                if dynamic {
                    self.ctx.use_core_helper(RuntimeHelper::NormalizeProps);
                    wrap_call("_normalizeProps", &object)
                } else {
                    object
                }
            }
            Segment::Spread(spread) => {
                self.ctx.use_core_helper(RuntimeHelper::NormalizeProps);
                self.ctx.use_core_helper(RuntimeHelper::GuardReactiveProps);
                wrap_call(
                    "_normalizeProps",
                    &wrap_call("_guardReactiveProps", &spread),
                )
            }
        }
    }

    /// One attached binding's prop entries; `Some` is a spread segment.
    fn component_binding(
        &mut self,
        binding: &s2::BindingOp<'_>,
        entries: &mut std::vec::Vec<VNodePropEntry>,
    ) -> Result<Option<String>> {
        match binding {
            s2::BindingOp::Bind(bind) => {
                let value = self.required_value(bind.value.as_ref())?;
                match &bind.name {
                    None => return Ok(Some(value)),
                    Some(DynamicName::Static(name)) => {
                        let key = bound_key(name, &bind.modifiers);
                        entries.push(component_prop_entry(&key, &value, false));
                    }
                    Some(name) => {
                        let key = dynamic_key(name)?.ok_or(LegacyReason::Binding)?;
                        entries.push(component_prop_entry(&key, &value, true));
                    }
                }
            }
            s2::BindingOp::On(on) => {
                let Some(name) = &on.name else {
                    let object = self.required_value(on.handler.as_ref())?;
                    self.ctx.use_core_helper(RuntimeHelper::ToHandlers);
                    return Ok(Some(wrap_call("_toHandlers", &object)));
                };
                let handler = match &on.handler {
                    Some(handler) => self.component_handler(handler)?,
                    None => "() => {}".to_compact_string(),
                };
                match name {
                    DynamicName::Static(name) => {
                        let key = vize_atelier_core::steps::create_on_name(name);
                        entries.push(component_prop_entry(&key, &handler, false));
                    }
                    DynamicName::Dynamic(_) => {
                        let name = dynamic_key(name)?.ok_or(LegacyReason::Binding)?;
                        self.ctx.use_core_helper(RuntimeHelper::ToHandlerKey);
                        let key = cstr!("_toHandlerKey({name})");
                        entries.push(component_prop_entry(&key, &handler, true));
                    }
                }
            }
            s2::BindingOp::Model(model) => {
                let value = self.expr(&model.contract.read, TransformContent::Decoded)?;
                let key = match &model.argument {
                    None => "modelValue".to_compact_string(),
                    Some(DynamicName::Static(name)) => camelize(name),
                    Some(DynamicName::Dynamic(_)) => return Err(LegacyReason::Binding.into()),
                };
                let update_key = cstr!("onUpdate:{key}");
                entries.push(component_prop_entry(&key, &value, false));
                let handler = cstr!("$event => (({value}) = $event)");
                entries.push(component_prop_entry(&update_key, &handler, false));
            }
            s2::BindingOp::VueShow(show) => {
                let exp = self.expr(&show.value, TransformContent::Decoded)?;
                entries.push(component_prop_entry(
                    "style",
                    &cstr!("(({exp}) ? null : {{ display: \"none\" }})"),
                    false,
                ));
            }
            _ => {}
        }
        Ok(None)
    }

    fn required_value(&self, value: Option<&ExprRef<'_>>) -> Result<String> {
        let value = value.ok_or(LegacyReason::Binding)?;
        self.expr(value, TransformContent::Decoded)
    }

    /// `event_handler_to_string` over the transform's processed handler.
    fn component_handler(&mut self, handler: &ExprRef<'_>) -> Result<String> {
        let processed = self
            .exprs
            .handler(handler, TransformContent::Decoded)
            .map_err(|_| LegacyReason::ExpressionOrEncoding)?;
        let rendered = self.consume(processed)?;
        if TransformExpressions::handler_is_callable(&rendered) {
            return Ok(rendered);
        }
        Ok(cstr!("$event => ({rendered})"))
    }
}

/// A spread or a dynamic key closes the current entry segment.
fn is_boundary(binding: &s2::BindingOp<'_>) -> bool {
    match binding {
        s2::BindingOp::Bind(bind) => !matches!(bind.name, Some(DynamicName::Static(_))),
        s2::BindingOp::On(on) => !matches!(on.name, Some(DynamicName::Static(_))),
        s2::BindingOp::Model(model) => matches!(model.argument, Some(DynamicName::Dynamic(_))),
        _ => false,
    }
}

fn flush(segments: &mut std::vec::Vec<Segment>, entries: &mut std::vec::Vec<VNodePropEntry>) {
    if !entries.is_empty() {
        segments.push(Segment::Entries(std::mem::take(entries)));
    }
}
