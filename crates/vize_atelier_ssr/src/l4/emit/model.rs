//! Native `v-model` realizations: `checked` / `value` attributes on inputs,
//! the dynamic-type helper, and `selected` on options of a modelled select.

use vize_atelier_core::RuntimeHelper;
use vize_l0::{String, ToCompactString, cstr};
use vize_l1_to_l2::{TransformContent, decode_template_entities};
use vize_l2::op::{self as l2, DynamicName};

use super::attrs::{Attached, admit_value, bound_value, static_value};
use super::{Emitter, Result};
use crate::codegen::element::VNodePropEntry;
use crate::codegen::element::props::{component_prop_entry, quoted_js_string};
use crate::l4::string_plan::SsrSegmentSource as Source;

/// `v-model` reads the plan emitter can own.
///
/// An argument on a plain element is `VModelArgOnElement`: the walker
/// reports it and emits no attribute. A read that is only a `v-for` or
/// slot alias is `VModelOnScope` and is dropped the same way. Both stay
/// on this lane; [`dropped`] writes nothing for them. On a non-form tag
/// the walker emits no attribute either.
pub(super) fn admit(model: &l2::ModelOp<'_>, _tag: &str) -> Result<()> {
    if dropped_read(model) {
        return Ok(());
    }
    admit_value(Some(&model.contract.read))
}

/// The walker removes this model before codegen.
pub(super) fn dropped(em: &Emitter<'_, '_, '_, '_, '_, '_>, model: &l2::ModelOp<'_>) -> bool {
    dropped_read(model) || alias_model(em, model)
}

fn dropped_read(model: &l2::ModelOp<'_>) -> bool {
    model.argument.is_some()
}

fn alias_model(em: &Emitter<'_, '_, '_, '_, '_, '_>, model: &l2::ModelOp<'_>) -> bool {
    let name = model.contract.read.source().trim();
    !name.is_empty() && em.scoped_params.iter().any(|frame| frame.contains(name))
}

impl Emitter<'_, '_, '_, '_, '_, '_> {
    fn model_read(&self, model: &l2::ModelOp<'_>) -> Result<String> {
        self.expr(&model.contract.read, TransformContent::Decoded)
    }

    /// The `:type` value, rewritten like every other bind value.
    fn dynamic_type(&self, attached: &Attached<'_, '_>) -> Result<Option<String>> {
        bound_value(attached, "type")
            .map(|value| self.expr(value, TransformContent::Decoded))
            .transpose()
    }

    fn model_attr_value(&self, attached: &Attached<'_, '_>, name: &str) -> Result<Option<String>> {
        for segment in attached {
            match segment.source {
                Source::Attribute(attr) if attr.name == name => {
                    if let Some(value) = attr.value {
                        return Ok(Some(quoted_js_string(&decode_template_entities(value))));
                    }
                }
                Source::Binding(l2::BindingOp::Bind(bind)) => {
                    if matches!(bind.name, Some(DynamicName::Static(bound)) if bound == name)
                        && let Some(value) = &bind.value
                    {
                        return self.expr(value, TransformContent::Decoded).map(Some);
                    }
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn checkbox_model_checked(&mut self, attached: &Attached<'_, '_>, exp: &str) -> Result<String> {
        if let Some(value) = self.model_attr_value(attached, "true-value")? {
            self.ctx.use_ssr_helper(RuntimeHelper::SsrLooseEqual);
            self.ctx.use_ssr_helper(RuntimeHelper::SsrLooseContain);
            let input_value = self
                .model_attr_value(attached, "value")?
                .unwrap_or_else(|| "null".to_compact_string());
            Ok(cstr!(
                "Array.isArray({exp}) ? _ssrLooseContain({exp}, {input_value}) : _ssrLooseEqual({exp}, {value})"
            ))
        } else {
            self.ctx.use_ssr_helper(RuntimeHelper::SsrLooseContain);
            let value = self
                .model_attr_value(attached, "value")?
                .unwrap_or_else(|| "null".to_compact_string());
            Ok(cstr!(
                "Array.isArray({exp}) ? _ssrLooseContain({exp}, {value}) : {exp}"
            ))
        }
    }
}

/// `process_v_model_on_element` at the directive's authored position.
pub(super) fn emit_inline(
    em: &mut Emitter<'_, '_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
    model: &l2::ModelOp<'_>,
    tag: &str,
) -> Result<()> {
    if dropped(em, model) {
        return Ok(());
    }
    let exp = em.model_read(model)?;
    match tag {
        "input" => {
            if let Some(type_exp) = em.dynamic_type(attached)? {
                em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderDynamicModel);
                let value = em
                    .model_attr_value(attached, "value")?
                    .unwrap_or_else(|| "null".to_compact_string());
                em.ctx.push_string_part_dynamic(&cstr!(
                    "_ssrRenderDynamicModel({type_exp}, {exp}, {value})"
                ));
                return Ok(());
            }
            match static_value(attached, "type").as_deref() {
                Some("checkbox") => {
                    em.ctx.use_ssr_helper(RuntimeHelper::SsrIncludeBooleanAttr);
                    let checked = em.checkbox_model_checked(attached, &exp)?;
                    em.ctx.push_string_part_dynamic(&cstr!(
                        "(_ssrIncludeBooleanAttr({checked})) ? \" checked\" : \"\""
                    ));
                }
                Some("radio") => {
                    em.ctx.use_ssr_helper(RuntimeHelper::SsrIncludeBooleanAttr);
                    em.ctx.use_ssr_helper(RuntimeHelper::SsrLooseEqual);
                    let value = em
                        .model_attr_value(attached, "value")?
                        .unwrap_or_else(|| "null".to_compact_string());
                    em.ctx.push_string_part_dynamic(&cstr!(
                        "(_ssrIncludeBooleanAttr(_ssrLooseEqual({exp}, {value}))) ? \" checked\" : \"\""
                    ));
                }
                _ => {
                    em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderAttr);
                    em.ctx
                        .push_string_part_dynamic(&cstr!("_ssrRenderAttr(\"value\", {exp})"));
                }
            }
        }
        "textarea" => em.ctx.use_ssr_helper(RuntimeHelper::SsrInterpolate),
        _ => {}
    }
    Ok(())
}

/// `collect_v_model_element_attr` on the fallthrough root: a prop entry, or
/// the read handed to `_ssrGetDynamicModelProps` for a dynamic `:type`.
pub(super) fn collect_root(
    em: &mut Emitter<'_, '_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
    model: &l2::ModelOp<'_>,
    tag: &str,
    entries: &mut std::vec::Vec<VNodePropEntry>,
    dynamic_model: &mut Option<String>,
) -> Result<()> {
    if dropped(em, model) {
        return Ok(());
    }
    let exp = em.model_read(model)?;
    if tag != "input" {
        return Ok(());
    }
    if bound_value(attached, "type").is_some() {
        *dynamic_model = Some(exp);
        return Ok(());
    }
    match static_value(attached, "type").as_deref() {
        Some("checkbox") => {
            let checked = em.checkbox_model_checked(attached, &exp)?;
            entries.push(component_prop_entry(
                "checked",
                &cstr!("({checked})"),
                false,
            ));
        }
        Some("radio") => {
            em.ctx.use_ssr_helper(RuntimeHelper::SsrLooseEqual);
            let value = em
                .model_attr_value(attached, "value")?
                .unwrap_or_else(|| "null".to_compact_string());
            entries.push(component_prop_entry(
                "checked",
                &cstr!("_ssrLooseEqual({exp}, {value})"),
                false,
            ));
        }
        _ => entries.push(component_prop_entry("value", &exp, false)),
    }
    Ok(())
}

/// `selected` on an `<option>` inside a `<select v-model>`.
pub(super) fn emit_option_selected(
    em: &mut Emitter<'_, '_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
) -> Result<()> {
    let Some(model) = em.select_models.last().cloned() else {
        return Ok(());
    };
    em.ctx.use_ssr_helper(RuntimeHelper::SsrIncludeBooleanAttr);
    em.ctx.use_ssr_helper(RuntimeHelper::SsrLooseContain);
    em.ctx.use_ssr_helper(RuntimeHelper::SsrLooseEqual);
    let value = em
        .model_attr_value(attached, "value")?
        .unwrap_or_else(|| "null".to_compact_string());
    em.ctx.push_string_part_dynamic(&cstr!(
        "((_ssrIncludeBooleanAttr(Array.isArray({model}) ? _ssrLooseContain({model}, {value}) : _ssrLooseEqual({model}, {value}))) ? \" selected\" : \"\")"
    ));
    Ok(())
}
