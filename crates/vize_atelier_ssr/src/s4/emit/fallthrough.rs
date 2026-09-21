//! The single fallthrough root: every attribute joins one props object that
//! merges with the component's `_attrs` (`build_element_attrs_expression`).

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, cstr};
use vize_s1_to_s2::{TransformContent, decode_template_entities};
use vize_s2::op as s2;

use super::attrs::{Attached, bind};
use super::{Emitter, Result, model, plan_source};
use crate::codegen::element::props::{
    component_prop_entry, component_props_object, normalize_prop_entries, quoted_js_string,
    wrap_call,
};
use crate::s4::string_plan::{SsrSegmentSource as Source, SsrStringPayloadKind};

pub(super) fn emit(
    em: &mut Emitter<'_, '_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
    tag: &str,
) -> Result<()> {
    let mut entries = std::vec::Vec::new();
    let mut spreads = std::vec::Vec::new();
    let mut dynamic_model = None;
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
            Source::Binding(binding) => match binding {
                s2::BindingOp::Bind(_) => {
                    let Some(bind) = bind(binding)? else {
                        continue;
                    };
                    let value = em.expr(bind.value, TransformContent::Decoded)?;
                    match bind.name {
                        Some(name) => entries.push(component_prop_entry(name, &value, false)),
                        None => spreads.push(value),
                    }
                }
                s2::BindingOp::Model(model) => {
                    model::collect_root(em, attached, model, tag, &mut entries, &mut dynamic_model)?
                }
                s2::BindingOp::VueShow(show) => {
                    let exp = em.expr(&show.value, TransformContent::Decoded)?;
                    entries.push(component_prop_entry(
                        "style",
                        &cstr!("(({exp}) ? null : {{ display: \"none\" }})"),
                        false,
                    ));
                }
                _ => {}
            },
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
    if let Some(model_exp) = dynamic_model {
        em.ctx
            .use_ssr_helper(RuntimeHelper::SsrGetDynamicModelProps);
        let existing = merge_args(em, &args);
        args.push(cstr!("_ssrGetDynamicModelProps({existing}, {model_exp})"));
    }

    let attrs = merge_args(em, &args);
    em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderAttrs);
    em.ctx
        .push_string_part_dynamic(&cstr!("_ssrRenderAttrs({attrs})"));
    Ok(())
}

/// One argument stays itself; several merge through `_mergeProps`.
fn merge_args(em: &mut Emitter<'_, '_, '_, '_, '_, '_>, args: &[String]) -> String {
    if let [only] = args {
        return only.clone();
    }
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
}
