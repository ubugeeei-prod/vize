//! Native `v-model` realizations: `checked` / `value` attributes on inputs,
//! the dynamic-type helper, and `selected` on options of a modelled select.

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, cstr};
use vize_s1_to_s2::TransformContent;
use vize_s2::op as s2;

use super::attrs::{Attached, admit_value, bound_value, static_value};
use super::{Emitter, Result};
use crate::codegen::element::VNodePropEntry;
use crate::codegen::element::props::{component_prop_entry, quoted_js_string};
use crate::s4::LegacyReason;

/// `v-model` reads the plan emitter can own.
///
/// An argument stays on the legacy walker (`<input v-model:foo>`). On a
/// non-form tag the legacy walker emits no attribute
/// (`process_v_model_on_element`'s empty arm); admitting the read lets the
/// plan do the same, and form tags still realize the read later.
pub(super) fn admit(model: &s2::ModelOp<'_>, _tag: &str) -> Result<()> {
    if model.argument.is_some() {
        return Err(LegacyReason::Binding.into());
    }
    admit_value(Some(&model.contract.read))
}

impl Emitter<'_, '_, '_, '_, '_, '_> {
    fn model_read(&self, model: &s2::ModelOp<'_>) -> Result<String> {
        self.expr(&model.contract.read, TransformContent::Decoded)
    }

    /// The `:type` value, rewritten like every other bind value.
    fn dynamic_type(&self, attached: &Attached<'_, '_>) -> Result<Option<String>> {
        bound_value(attached, "type")
            .map(|value| self.expr(value, TransformContent::Decoded))
            .transpose()
    }
}

/// `process_v_model_on_element` at the directive's authored position.
pub(super) fn emit_inline(
    em: &mut Emitter<'_, '_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
    model: &s2::ModelOp<'_>,
    tag: &str,
) -> Result<()> {
    let exp = em.model_read(model)?;
    match tag {
        "input" => {
            if let Some(type_exp) = em.dynamic_type(attached)? {
                em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderDynamicModel);
                em.ctx.push_string_part_dynamic(&cstr!(
                    "_ssrRenderDynamicModel({type_exp}, {exp}, null)"
                ));
                return Ok(());
            }
            match static_value(attached, "type").as_deref() {
                Some("checkbox") => {
                    em.ctx.use_ssr_helper(RuntimeHelper::SsrIncludeBooleanAttr);
                    em.ctx.use_ssr_helper(RuntimeHelper::SsrLooseContain);
                    em.ctx.push_string_part_dynamic(&cstr!(
                        "(_ssrIncludeBooleanAttr(Array.isArray({exp}) ? _ssrLooseContain({exp}, null) : {exp})) ? \" checked\" : \"\""
                    ));
                }
                Some("radio") => {
                    em.ctx.use_ssr_helper(RuntimeHelper::SsrIncludeBooleanAttr);
                    em.ctx.use_ssr_helper(RuntimeHelper::SsrLooseEqual);
                    let value = static_value(attached, "value")
                        .map(|value| quoted_js_string(&value))
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
    model: &s2::ModelOp<'_>,
    tag: &str,
    entries: &mut std::vec::Vec<VNodePropEntry>,
    dynamic_model: &mut Option<String>,
) -> Result<()> {
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
            em.ctx.use_ssr_helper(RuntimeHelper::SsrLooseContain);
            entries.push(component_prop_entry(
                "checked",
                &cstr!("(Array.isArray({exp}) ? _ssrLooseContain({exp}, null) : {exp})"),
                false,
            ));
        }
        Some("radio") => {
            em.ctx.use_ssr_helper(RuntimeHelper::SsrLooseEqual);
            let value = static_value(attached, "value")
                .map(|value| quoted_js_string(&value))
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
    let value = match static_value(attached, "value") {
        Some(value) => quoted_js_string(&value),
        None => match bound_value(attached, "value") {
            Some(value) => em.expr(value, TransformContent::Decoded)?,
            None => "null".to_compact_string(),
        },
    };
    em.ctx.push_string_part_dynamic(&cstr!(
        "((_ssrIncludeBooleanAttr(Array.isArray({model}) ? _ssrLooseContain({model}, {value}) : _ssrLooseEqual({model}, {value}))) ? \" selected\" : \"\")"
    ));
    Ok(())
}
