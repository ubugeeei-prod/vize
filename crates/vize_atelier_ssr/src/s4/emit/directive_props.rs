//! `_ssrGetDirectiveProps(_ctx, dir[, value[, arg[, modifiers]]])` for a
//! custom directive on a merged element (`buildDirectiveArgs` padding).

use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, cstr};
use vize_s1_to_s2::TransformContent;
use vize_s2::op::{self as s2, DynamicName};

use super::{Emitter, Result};
use crate::codegen::element::props::{is_valid_js_identifier, quoted_js_string};

impl Emitter<'_, '_, '_, '_, '_, '_> {
    /// `_ssrGetDirectiveProps(_ctx, dir[, value[, arg[, modifiers]]])`.
    pub(super) fn directive_props(&mut self, directive: &s2::VueDirectiveOp<'_>) -> Result<String> {
        self.ctx.use_ssr_helper(RuntimeHelper::SsrGetDirectiveProps);
        let reference = self.ctx.directive_reference(directive.name);
        let mut out = cstr!("_ssrGetDirectiveProps(_ctx, {reference}");
        let value = directive
            .value
            .as_ref()
            .map(|value| self.expr(value, TransformContent::Decoded))
            .transpose()?;
        if let Some(value) = &value {
            out.push_str(", ");
            out.push_str(value);
        }
        let argument = match &directive.argument {
            None => None,
            Some(DynamicName::Static(name)) => Some(quoted_js_string(name)),
            Some(name) => Some(self.directive_argument(name)?),
        };
        if let Some(argument) = &argument {
            if value.is_none() {
                out.push_str(", void 0");
            }
            out.push_str(", ");
            out.push_str(argument);
        }
        if !directive.modifiers.is_empty() {
            if argument.is_none() {
                if value.is_none() {
                    out.push_str(", void 0");
                }
                out.push_str(", void 0");
            }
            let flags = directive
                .modifiers
                .iter()
                .map(|modifier| {
                    let key = if is_valid_js_identifier(modifier) {
                        modifier.to_compact_string()
                    } else {
                        quoted_js_string(modifier)
                    };
                    cstr!("{key}: true")
                })
                .collect::<std::vec::Vec<_>>();
            out.push_str(&cstr!(", {{ {} }}", flags.join(", ")));
        }
        out.push(')');
        Ok(out)
    }
}
