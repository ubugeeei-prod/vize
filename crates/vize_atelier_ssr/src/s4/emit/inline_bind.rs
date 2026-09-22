//! One non-root `v-bind` rendered as an inline attribute part (the legacy
//! `process_directive_on_element` bind arm): `class` / `style` merges, boolean
//! attributes, and `_ssrRenderAttr` with the name anchored at the authored
//! argument.

use vize_atelier_core::RuntimeHelper;
use vize_atelier_core::codegen::spanned::SpannedText;
use vize_s0::{String, cstr};

use super::attrs::{Attached, Bind, BindName, static_value};
use super::spans::argument_start;
use super::{Emitter, Result};
use crate::codegen::element::props::{merge_prop_values, quoted_js_string};

pub(super) fn emit(
    em: &mut Emitter<'_, '_, '_, '_, '_, '_>,
    attached: &Attached<'_, '_>,
    bind: &Bind<'_, '_>,
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
            // The name maps to the authored argument and the value to its
            // expression, as the AST walker writes `_ssrRenderAttr`.
            let mut piece = SpannedText::plain("_ssrRenderAttr(\"");
            // A camelized name maps to its authored (kebab) argument, as the
            // walker anchors the name at the argument whatever its spelling.
            let authored = match bind.name {
                BindName::Static(authored) => authored,
                _ => name,
            };
            match argument_start(em.ctx.source, bind.span, authored) {
                Some(start) if em.ctx.spans_enabled() => piece.push_mapped(name, start),
                _ => piece.push_str(name),
            }
            piece.push_str("\", ");
            piece.push_spanned(&em.bound_expression(&exp, bind));
            piece.push_str(")");
            em.ctx.push_string_part_dynamic_spanned(piece);
        }
        None => {
            em.ctx.use_ssr_helper(RuntimeHelper::SsrRenderAttrs);
            em.ctx
                .push_string_part_dynamic(&cstr!("_ssrRenderAttrs({exp})"));
        }
    }
    Ok(())
}
