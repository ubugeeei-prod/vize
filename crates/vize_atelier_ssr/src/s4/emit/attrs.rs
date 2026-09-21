//! Element attribute output: the inline `name="value"` string shape and the
//! fallthrough-root `_ssrRenderAttrs(_mergeProps(...))` object shape.

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, cstr};
use vize_s1_to_s2::{TransformContent, decode_template_entities};
use vize_s2::expr::ExprRef;
use vize_s2::op::{self as s2, DynamicName};

use super::{Emitter, Result, plan_source, require_dynamic};
use crate::codegen::element::props::{
    component_prop_entry, component_props_object, merge_prop_values, normalize_prop_entries,
    quoted_js_string, wrap_call,
};
use crate::codegen::helpers::escape_html_attr;
use crate::s4::string_plan::{
    SsrSegmentSource as Source, SsrStringPayloadKind, SsrStringSegment,
    SsrStringSegmentKind as Kind,
};
use crate::s4::{AdmissionFailure, LegacyReason};

type Attached<'r, 'a> = [SsrStringSegment<'r, 'a>];

/// One admitted `v-bind`: a static name or the object spread, never modified.
struct Bind<'r, 'a> {
    name: Option<&'a str>,
    value: &'r ExprRef<'a>,
}

/// Admit an element's attached segments: static attributes, plain `v-bind`,
/// and `v-on` (which SSR drops). Anything else keeps the legacy lane.
pub(super) fn admit(attached: &Attached<'_, '_>, owner_fact: u32) -> Result<()> {
    if attached
        .windows(2)
        .any(|pair| pair[0].span.start > pair[1].span.start)
    {
        return Err(AdmissionFailure::Invalid(
            "string plan attached segments are not in authored order",
        ));
    }
    for segment in attached {
        match (segment.kind, segment.source) {
            (Kind::StaticAttribute, Source::Attribute(_)) if segment.fact == owner_fact => {}
            (_, Source::Binding(binding)) => {
                require_dynamic(segment)?;
                match binding {
                    s2::BindingOp::Bind(_) => {
                        bind(binding)?;
                    }
                    s2::BindingOp::On(_) => {}
                    _ => return Err(LegacyReason::Binding.into()),
                }
            }
            _ => {
                return Err(AdmissionFailure::Invalid(
                    "string plan attached segment does not belong to its element",
                ));
            }
        }
    }
    Ok(())
}

fn bind<'r, 'a>(binding: &'r s2::BindingOp<'a>) -> Result<Option<Bind<'r, 'a>>> {
    let s2::BindingOp::Bind(bind) = binding else {
        return Ok(None);
    };
    if !bind.modifiers.is_empty() {
        return Err(LegacyReason::Binding.into());
    }
    let name = match bind.name {
        None => None,
        Some(DynamicName::Static(name)) => Some(name),
        Some(DynamicName::Dynamic(_)) => return Err(LegacyReason::Binding.into()),
    };
    match &bind.value {
        // An opaque value (entity-encoded source the S2 parse refused) still
        // reaches the transform rewrite, which re-parses the decoded text and
        // refuses exactly where the shipped transform reports it invalid.
        Some(value @ (ExprRef::Js(_) | ExprRef::Opaque(_))) => Ok(Some(Bind { name, value })),
        Some(_) => Err(LegacyReason::ExpressionOrEncoding.into()),
        None => Err(LegacyReason::Binding.into()),
    }
}

impl Emitter<'_, '_, '_, '_, '_> {
    /// The `v-bind` value exactly as the shipped transform rewrote it.
    fn bind_value(&self, value: &ExprRef<'_>) -> Result<String> {
        let rewritten = self
            .exprs
            .expr(value, TransformContent::Decoded)
            .map_err(|_| LegacyReason::ExpressionOrEncoding)?;
        if rewritten.used_unref {
            return Err(LegacyReason::ExpressionOrEncoding.into());
        }
        Ok(rewritten.text)
    }
}

/// The first static attribute named `name`, entity-decoded.
fn static_value(attached: &Attached<'_, '_>, name: &str) -> Option<String> {
    let attr = attached.iter().find_map(|segment| match segment.source {
        Source::Attribute(attr) if attr.name == name => Some(attr),
        _ => None,
    })?;
    attr.value.map(decode_template_entities)
}

fn has_bound(attached: &Attached<'_, '_>, name: &str) -> bool {
    attached.iter().any(|segment| match segment.source {
        Source::Binding(s2::BindingOp::Bind(bind)) => {
            matches!(bind.name, Some(DynamicName::Static(bound)) if bound == name)
        }
        _ => false,
    })
}

/// Non-root element attributes, in authored order.
pub(super) fn emit_inline(
    em: &mut Emitter<'_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
) -> Result<()> {
    let dynamic_class = has_bound(attached, "class");
    let dynamic_style = has_bound(attached, "style");
    for segment in attached {
        match segment.source {
            Source::Attribute(attr) => {
                let name = plan_source(segment, SsrStringPayloadKind::AttributeName)?;
                if (name == "class" && dynamic_class)
                    || (name == "style" && dynamic_style)
                    || vize_s0::is_reserved_prop(name)
                {
                    continue;
                }
                em.ctx.push_string_part_static(" ");
                em.ctx.push_string_part_static(name);
                if let Some(value) = attr.value {
                    em.ctx.push_string_part_static("=\"");
                    let decoded = decode_template_entities(value);
                    em.ctx.push_string_part_static(&escape_html_attr(&decoded));
                    em.ctx.push_string_part_static("\"");
                }
            }
            Source::Binding(binding) => {
                let Some(bind) = bind(binding)? else {
                    continue;
                };
                let exp = em.bind_value(bind.value)?;
                emit_inline_bind(em, attached, bind.name, exp);
            }
            _ => {}
        }
    }
    Ok(())
}

fn emit_inline_bind(
    em: &mut Emitter<'_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
    name: Option<&str>,
    exp: String,
) {
    match name {
        Some(name) if vize_s0::is_reserved_prop(name) => {}
        Some("class") => {
            em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderClass);
            em.ctx.push_string_part_static(" class=\"");
            let class_exp = match static_value(attached, "class") {
                Some(static_class) => {
                    let quoted = quoted_js_string(&static_class);
                    cstr!("_ssrRenderClass([{quoted}, {exp}])")
                }
                None => cstr!("_ssrRenderClass({exp})"),
            };
            em.ctx.push_string_part_dynamic(&class_exp);
            em.ctx.push_string_part_static("\"");
        }
        Some("style") => {
            em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderStyle);
            em.ctx.push_string_part_static(" style=\"");
            let mut values = std::vec::Vec::new();
            if let Some(static_style) = static_value(attached, "style") {
                values.push(quoted_js_string(&static_style));
            }
            values.push(exp);
            let style_exp = merge_prop_values(values);
            em.ctx
                .push_string_part_dynamic(&cstr!("_ssrRenderStyle({style_exp})"));
            em.ctx.push_string_part_static("\"");
        }
        Some(name) if vize_s0::is_boolean_attr(name) => {
            em.ctx.use_ssr_helper(RuntimeHelper::SsrIncludeBooleanAttr);
            em.ctx.push_string_part_dynamic(&cstr!(
                "(_ssrIncludeBooleanAttr({exp})) ? \" {name}\" : \"\""
            ));
        }
        Some(name) => {
            em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderAttr);
            em.ctx
                .push_string_part_dynamic(&cstr!("_ssrRenderAttr(\"{name}\", {exp})"));
        }
        None => {
            em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderAttrs);
            em.ctx
                .push_string_part_dynamic(&cstr!("_ssrRenderAttrs({exp})"));
        }
    }
}

/// The single fallthrough root: every attribute joins one props object that
/// merges with the component's `_attrs`.
pub(super) fn emit_fallthrough(
    em: &mut Emitter<'_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
) -> Result<()> {
    let mut entries = std::vec::Vec::new();
    let mut spreads = std::vec::Vec::new();
    for segment in attached {
        match segment.source {
            Source::Attribute(attr) => {
                let value = attr
                    .value
                    .map(|value| quoted_js_string(&decode_template_entities(value)))
                    .unwrap_or_else(|| "\"\"".to_compact_string());
                let name = plan_source(segment, SsrStringPayloadKind::AttributeName)?;
                entries.push(component_prop_entry(name, &value, false));
            }
            Source::Binding(binding) => {
                let Some(bind) = bind(binding)? else {
                    continue;
                };
                let value = em.bind_value(bind.value)?;
                match bind.name {
                    Some(name) => entries.push(component_prop_entry(name, &value, false)),
                    None => spreads.push(value),
                }
            }
            _ => {}
        }
    }

    let entries = normalize_prop_entries(entries);
    let mut args: std::vec::Vec<String> = std::vec::Vec::new();
    if !spreads.is_empty() {
        em.ctx.use_core_helper(RuntimeHelper::NormalizeProps);
        em.ctx.use_core_helper(RuntimeHelper::GuardReactiveProps);
        args.extend(
            spreads.iter().map(|spread| {
                wrap_call("_normalizeProps", &wrap_call("_guardReactiveProps", spread))
            }),
        );
    }
    if !entries.is_empty() {
        args.push(component_props_object(&entries));
    }
    args.push("_attrs".to_compact_string());

    let attrs = if let [only] = args.as_slice() {
        only.clone()
    } else {
        em.ctx.use_core_helper(RuntimeHelper::MergeProps);
        let mut out = String::from("_mergeProps(");
        for (index, arg) in args.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(arg);
        }
        out.push(')');
        out
    };
    em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderAttrs);
    em.ctx
        .push_string_part_dynamic(&cstr!("_ssrRenderAttrs({attrs})"));
    Ok(())
}
