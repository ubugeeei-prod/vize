//! SSR input property and model expressions.

use super::{
    ElementNode, ExpressionNode, PropNode, RuntimeHelper, SsrCodegenContext, String,
    ToCompactString, cstr, quoted_js_string,
};

impl<'a> SsrCodegenContext<'a> {
    /// Get an attribute value from an element
    pub(crate) fn get_element_attr_value(&self, el: &ElementNode, name: &str) -> Option<String> {
        use vize_atelier_core::PropNode;

        for prop in &el.props {
            if let PropNode::Attribute(attr) = prop
                && attr.name == name
            {
                return attr.value.as_ref().map(|v| v.content.to_compact_string());
            }
        }
        None
    }

    /// Form model comparisons consume the bound expression, or the quoted
    /// static attribute, rather than the serialized HTML attribute value.
    pub(super) fn model_attr_value(&mut self, el: &ElementNode, name: &str) -> Option<String> {
        self.get_dynamic_bind_exp(el, name).or_else(|| {
            self.get_element_attr_value(el, name)
                .map(|value| quoted_js_string(&value))
        })
    }

    pub(super) fn checkbox_model_checked(&mut self, el: &ElementNode, exp: &str) -> String {
        if let Some(value) = self.model_attr_value(el, "true-value") {
            self.use_ssr_helper(RuntimeHelper::SsrLooseEqual);
            cstr!("_ssrLooseEqual({exp}, {value})")
        } else {
            self.use_ssr_helper(RuntimeHelper::SsrLooseContain);
            let value = self
                .model_attr_value(el, "value")
                .unwrap_or_else(|| "null".to_compact_string());
            cstr!("Array.isArray({exp}) ? _ssrLooseContain({exp}, {value}) : {exp}")
        }
    }

    /// Return the source expression bound by `:name` (or `v-bind:name`) on
    /// `el`, if any. Used by SSR v-model lowering to find `:type` on
    /// `<input :type="t" v-model>` so the dynamic-model helper kicks in.
    /// (#962)
    pub(super) fn get_dynamic_bind_exp(&mut self, el: &ElementNode, name: &str) -> Option<String> {
        for prop in &el.props {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            if dir.name != "bind" {
                continue;
            }
            let matches_name = matches!(
                &dir.arg,
                Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == name
            );
            if matches_name && let Some(exp) = &dir.exp {
                return Some(self.expression_to_string(exp));
            }
        }
        None
    }

    pub(super) fn has_dynamic_bind(&self, el: &ElementNode, name: &str) -> bool {
        el.props.iter().any(|prop| {
            let PropNode::Directive(dir) = prop else {
                return false;
            };
            if dir.name != "bind" {
                return false;
            }
            matches!(&dir.arg, Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == name)
        })
    }
}
