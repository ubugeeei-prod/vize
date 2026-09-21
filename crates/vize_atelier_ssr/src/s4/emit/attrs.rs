//! Attached-segment admission and the inline `name="value"` attribute shape
//! of non-root elements.

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, cstr};
use vize_s1_to_s2::{TransformContent, decode_template_entities};
use vize_s2::expr::ExprRef;
use vize_s2::op::{self as s2, DynamicName};

use super::{Emitter, Result, model, plan_source, require_dynamic};
use crate::codegen::element::props::{merge_prop_values, quoted_js_string};
use crate::codegen::helpers::escape_html_attr;
use crate::s4::string_plan::{
    SsrSegmentSource as Source, SsrStringPayloadKind, SsrStringSegment,
    SsrStringSegmentKind as Kind,
};
use crate::s4::{AdmissionFailure, LegacyReason};

pub(super) type Attached<'r, 'a> = [SsrStringSegment<'r, 'a>];

/// One admitted `v-bind`: a static name or the object spread, never modified.
pub(super) struct Bind<'r, 'a> {
    pub(super) name: Option<&'a str>,
    pub(super) value: &'r ExprRef<'a>,
}

/// Admit an element's attached segments. Anything the plan emitter does not
/// own keeps the legacy lane; a segment that does not belong to its element
/// is a broken plan.
pub(super) fn admit(attached: &Attached<'_, '_>, owner_fact: u32, tag: &str) -> Result<()> {
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
                match binding {
                    s2::BindingOp::Bind(_) => {
                        bind(binding)?;
                    }
                    s2::BindingOp::On(_) => {}
                    s2::BindingOp::Model(model) => model::admit(model, tag)?,
                    s2::BindingOp::VueShow(show) => admit_value(Some(&show.value))?,
                    s2::BindingOp::VueHtml(html) => admit_value(html.value.as_ref())?,
                    s2::BindingOp::VueText(text) => admit_value(text.value.as_ref())?,
                    _ => return Err(LegacyReason::Binding.into()),
                }
                require_dynamic(segment)?;
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

/// A directive value the transform rewrite can own. An opaque value (the S2
/// parse refused the raw text) still reaches the rewrite, which re-parses and
/// refuses exactly where the shipped transform reports it invalid.
pub(super) fn admit_value(value: Option<&ExprRef<'_>>) -> Result<()> {
    match value {
        Some(ExprRef::Js(_) | ExprRef::Opaque(_)) => Ok(()),
        Some(_) => Err(LegacyReason::ExpressionOrEncoding.into()),
        None => Err(LegacyReason::Binding.into()),
    }
}

pub(super) fn bind<'r, 'a>(binding: &'r s2::BindingOp<'a>) -> Result<Option<Bind<'r, 'a>>> {
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
    admit_value(bind.value.as_ref())?;
    match &bind.value {
        Some(value) => Ok(Some(Bind { name, value })),
        None => Err(LegacyReason::Binding.into()),
    }
}

/// The first static attribute named `name`, entity-decoded (`None` when the
/// first such attribute is valueless, as the legacy lookup returns).
pub(super) fn static_value(attached: &Attached<'_, '_>, name: &str) -> Option<String> {
    let attr = attached.iter().find_map(|segment| match segment.source {
        Source::Attribute(attr) if attr.name == name => Some(attr),
        _ => None,
    })?;
    attr.value.map(decode_template_entities)
}

/// The first static-name `v-bind:name` value.
pub(super) fn bound_value<'r, 'a>(
    attached: &Attached<'r, 'a>,
    name: &str,
) -> Option<&'r ExprRef<'a>> {
    attached.iter().find_map(|segment| match segment.source {
        Source::Binding(s2::BindingOp::Bind(bind)) => match bind.name {
            Some(DynamicName::Static(bound)) if bound == name => bind.value.as_ref(),
            _ => None,
        },
        _ => None,
    })
}

fn has_bound(attached: &Attached<'_, '_>, name: &str) -> bool {
    attached.iter().any(|segment| match segment.source {
        Source::Binding(s2::BindingOp::Bind(bind)) => {
            matches!(bind.name, Some(DynamicName::Static(bound)) if bound == name)
        }
        _ => false,
    })
}

/// The first `v-show` value.
fn show_value<'r, 'a>(attached: &Attached<'r, 'a>) -> Option<&'r ExprRef<'a>> {
    attached.iter().find_map(|segment| match segment.source {
        Source::Binding(s2::BindingOp::VueShow(show)) => Some(&show.value),
        _ => None,
    })
}

impl Emitter<'_, '_, '_, '_, '_, '_> {
    /// `(({exp}) ? null : { display: "none" })` for the first `v-show`.
    pub(super) fn show_style(&self, attached: &Attached<'_, '_>) -> Result<Option<String>> {
        let Some(value) = show_value(attached) else {
            return Ok(None);
        };
        let exp = self.expr(value, TransformContent::Decoded)?;
        Ok(Some(cstr!("(({exp}) ? null : {{ display: \"none\" }})")))
    }
}

/// Non-root element attributes, in authored order.
pub(super) fn emit_inline(
    em: &mut Emitter<'_, '_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
    tag: &str,
) -> Result<()> {
    let dynamic_class = has_bound(attached, "class");
    let dynamic_style = has_bound(attached, "style");
    let explicit_style = dynamic_style
        || attached.iter().any(
            |segment| matches!(segment.source, Source::Attribute(attr) if attr.name == "style"),
        );
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
                if name == "style"
                    && let Some(show) = em.show_style(attached)?
                {
                    let value = attr
                        .value
                        .map(|value| quoted_js_string(&decode_template_entities(value)))
                        .unwrap_or_else(|| "\"\"".to_compact_string());
                    let style_exp = merge_prop_values(std::vec![value, show]);
                    em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderStyle);
                    em.ctx.push_string_part_static(" style=\"");
                    em.ctx
                        .push_string_part_dynamic(&cstr!("_ssrRenderStyle({style_exp})"));
                    em.ctx.push_string_part_static("\"");
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
            Source::Binding(binding) => match binding {
                s2::BindingOp::Bind(_) => {
                    let Some(bind) = bind(binding)? else {
                        continue;
                    };
                    let exp = em.expr(bind.value, TransformContent::Decoded)?;
                    emit_inline_bind(em, attached, bind.name, exp)?;
                }
                s2::BindingOp::Model(model) => model::emit_inline(em, attached, model, tag)?,
                s2::BindingOp::VueShow(show) if !explicit_style => {
                    let exp = em.expr(&show.value, TransformContent::Decoded)?;
                    em.ctx.push_string_part_dynamic(&cstr!(
                        "(({exp}) ? \"\" : \" style=\\\"display: none;\\\"\")"
                    ));
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}

fn emit_inline_bind(
    em: &mut Emitter<'_, '_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
    name: Option<&str>,
    exp: String,
) -> Result<()> {
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
            if let Some(show) = em.show_style(attached)? {
                values.push(show);
            }
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
    Ok(())
}
