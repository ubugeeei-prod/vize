//! The single fallthrough root: every attribute joins one props object that
//! merges with the component's `_attrs` (`build_element_attrs_expression`).

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, cstr};
use vize_s1_to_s2::{TransformContent, decode_template_entities};
use vize_s2::op as s2;

use super::attrs::{Attached, bind};
use super::spans::{argument_start, attribute_value_start};
use super::{Emitter, Result, model, plan_source};
use crate::codegen::element::VNodePropEntry;
use crate::codegen::element::props::{
    component_prop_entry, normalize_prop_entries, quoted_js_string, wrap_call,
};
use crate::codegen::element::spanned_props::{
    bound_entry, component_props_object_spanned, merge_props_call, wrap_spanned,
};
use crate::s4::string_plan::{SsrSegmentSource as Source, SsrStringPayloadKind};
use vize_atelier_core::codegen::document::EmitDocument;

pub(super) fn emit(
    em: &mut Emitter<'_, '_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
    tag: &str,
) -> Result<()> {
    let spans = em.ctx.spans_enabled();
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
                entries.push(if spans {
                    static_entry(em, attr, name, &value)
                } else {
                    component_prop_entry(name, &value, false)
                });
            }
            Source::Binding(binding) => match binding {
                s2::BindingOp::Bind(_) => {
                    let Some(bind) = bind(binding)? else {
                        continue;
                    };
                    let value = em.expr(bind.value, TransformContent::Decoded)?;
                    match bind.name {
                        Some(name) if spans => {
                            let key = argument_start(em.ctx.source, bind.span, name);
                            let spanned = em.bound_expression(&value, &bind);
                            entries.push(bound_entry(name, key, spanned));
                        }
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
    let mut args: std::vec::Vec<EmitDocument> = std::vec::Vec::new();
    if !spreads.is_empty() {
        em.ctx.use_core_helper(RuntimeHelper::NormalizeProps);
        em.ctx.use_core_helper(RuntimeHelper::GuardReactiveProps);
        args.extend(spreads.iter().map(|spread| {
            let guarded = wrap_call("_guardReactiveProps", spread);
            EmitDocument::from(wrap_call("_normalizeProps", &guarded))
        }));
    }
    if !entries.is_empty() {
        args.push(component_props_object_spanned(&entries));
    }
    args.push(EmitDocument::plain("_attrs"));
    if let Some(model_exp) = dynamic_model {
        em.ctx
            .use_ssr_helper(RuntimeHelper::SsrGetDynamicModelProps);
        let existing: String = merge_args(em, &args).as_str().into();
        let model = cstr!("_ssrGetDynamicModelProps({existing}, {model_exp})");
        args.push(EmitDocument::from(model));
    }

    let attrs = merge_args(em, &args);
    em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderAttrs);
    em.ctx
        .push_string_part_dynamic_spanned(wrap_spanned("_ssrRenderAttrs", &attrs));
    Ok(())
}

/// One argument stays itself; several merge through `_mergeProps`.
fn merge_args(em: &mut Emitter<'_, '_, '_, '_, '_, '_>, args: &[EmitDocument]) -> EmitDocument {
    if let [only] = args {
        return only.clone();
    }
    em.ctx.use_core_helper(RuntimeHelper::MergeProps);
    merge_props_call(args)
}

/// A static attribute's props-object entry with its key and value anchored
/// at the authored tokens, as the AST walker builds it.
fn static_entry(
    em: &Emitter<'_, '_, '_, '_, '_, '_>,
    attr: &s2::Attribute<'_>,
    name: &str,
    value: &str,
) -> VNodePropEntry {
    let start = attr
        .value
        .and_then(|_| attribute_value_start(em.ctx.source, attr.span, name));
    let spanned = match start {
        Some(start) if value.len() >= 2 => {
            let mut spanned = EmitDocument::plain("\"");
            spanned.push_mapped(&value[1..value.len() - 1], start);
            spanned.push_str("\"");
            spanned
        }
        _ => EmitDocument::plain(value),
    };
    bound_entry(name, Some(attr.span.start), spanned)
}
