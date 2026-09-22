//! `_ssrGetDirectiveProps(...)` for a custom directive on a merged element,
//! and the directive reference `resolveSetupReference` / `_resolveDirective`
//! pick (`@vue/compiler-ssr` 3.5's `buildDirectiveArgs`).

use super::props::quoted_js_string;
use super::{
    DirectiveNode, ExpressionNode, RuntimeHelper, SsrCodegenContext, String, ToCompactString, cstr,
};
use vize_atelier_core::BindingType;
use vize_s0::{camelize, capitalize};

impl SsrCodegenContext<'_> {
    /// `_ssrGetDirectiveProps(_ctx, dir[, value[, arg[, modifiers]]])` with
    /// `buildDirectiveArgs`' `void 0` padding.
    pub(super) fn directive_props(&mut self, dir: &DirectiveNode) -> String {
        self.use_ssr_helper(RuntimeHelper::SsrGetDirectiveProps);
        let mut out = cstr!(
            "_ssrGetDirectiveProps(_ctx, {}",
            self.directive_reference(dir.name)
        );
        let exp = dir.exp.as_ref().map(|exp| self.expression_to_string(exp));
        if let Some(exp) = &exp {
            out.push_str(", ");
            out.push_str(exp);
        }
        let arg = dir.arg.as_ref().map(|arg| match arg {
            ExpressionNode::Simple(simple) if simple.is_static => quoted_js_string(simple.content),
            _ => self.dynamic_arg_to_string(arg),
        });
        if let Some(arg) = &arg {
            if exp.is_none() {
                out.push_str(", void 0");
            }
            out.push_str(", ");
            out.push_str(arg);
        }
        if !dir.modifiers.is_empty() {
            if arg.is_none() {
                if exp.is_none() {
                    out.push_str(", void 0");
                }
                out.push_str(", void 0");
            }
            let flags = dir
                .modifiers
                .iter()
                .map(|modifier| {
                    let key = if super::props::is_valid_js_identifier(modifier.content) {
                        modifier.content.to_compact_string()
                    } else {
                        quoted_js_string(modifier.content)
                    };
                    cstr!("{key}: true")
                })
                .collect::<std::vec::Vec<_>>();
            out.push_str(&cstr!(", {{ {} }}", flags.join(", ")));
        }
        out.push(')');
        out
    }

    /// `resolveSetupReference("v-" + name)` over the script bindings, else
    /// `_resolveDirective(name)` (inlined, like component resolution).
    pub(crate) fn directive_reference(&mut self, name: &str) -> String {
        let camel = camelize(&cstr!("v-{name}"));
        let pascal = capitalize(&camel);
        let bound = self.options.binding_metadata.as_ref().and_then(|metadata| {
            [camel.as_str(), pascal.as_str()]
                .into_iter()
                .find_map(|candidate| {
                    metadata.bindings.get(candidate).and_then(|binding| {
                        matches!(
                            binding,
                            BindingType::SetupConst
                                | BindingType::SetupReactiveConst
                                | BindingType::LiteralConst
                                | BindingType::SetupLet
                                | BindingType::SetupRef
                                | BindingType::SetupMaybeRef
                        )
                        .then(|| (candidate.to_compact_string(), *binding))
                    })
                })
        });
        match bound {
            Some((binding, kind)) if self.options.inline => {
                if matches!(
                    kind,
                    BindingType::SetupLet | BindingType::SetupRef | BindingType::SetupMaybeRef
                ) {
                    self.use_core_helper(RuntimeHelper::Unref);
                    cstr!("_unref({binding})")
                } else {
                    binding
                }
            }
            Some((binding, _)) => cstr!("$setup[{}]", quoted_js_string(&binding)),
            None => {
                self.use_core_helper(RuntimeHelper::ResolveDirective);
                cstr!("_resolveDirective({})", quoted_js_string(name))
            }
        }
    }
}
