use vize_atelier_core::RuntimeHelper;
use vize_s0::{String, ToCompactString, camelize, capitalize};

use super::SsrCodegenContext;

impl SsrCodegenContext<'_> {
    pub(crate) fn is_self_component_reference(&self, component: &str) -> bool {
        let Some(component_name) = self.component_name.as_deref() else {
            return false;
        };
        if self.experimental_self_component && component == "Self" && !component_name.is_empty() {
            return true;
        }
        if component == component_name {
            return true;
        }
        let camel = camelize(component);
        let pascal = capitalize(camel.as_str());
        pascal == component_name
    }

    pub(crate) fn component_resolution_name(&self, component: &str) -> String {
        if self.experimental_self_component
            && component == "Self"
            && let Some(component_name) = self.component_name.as_deref()
            && !component_name.is_empty()
        {
            component_name.to_compact_string()
        } else {
            component.to_compact_string()
        }
    }

    pub(crate) fn resolved_component_callee(&mut self, component: &str) -> String {
        self.use_core_helper(RuntimeHelper::ResolveComponent);
        let mut out = String::from("_resolveComponent(");
        push_quoted_js_string(&mut out, self.component_resolution_name(component).as_str());
        if self.is_self_component_reference(component) {
            out.push_str(", true");
        }
        out.push(')');
        out
    }
}

fn push_quoted_js_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}
