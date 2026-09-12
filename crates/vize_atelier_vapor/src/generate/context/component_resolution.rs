use vize_carton::{String, ToCompactString, camelize, capitalize, cstr};

use super::GenerateContext;

impl<'a> GenerateContext<'a> {
    pub(crate) fn resolve_component_binding_expr(&self, component: &str) -> Option<String> {
        if self.experimental_self_component && component == "Self" {
            return None;
        }

        let bindings = self.binding_metadata?;
        let resolve_base = |name: &str| {
            if bindings.bindings.contains_key(name) {
                return Some(name.to_compact_string());
            }
            let camel = camelize(name);
            if bindings.bindings.contains_key(camel.as_str()) {
                return Some(camel);
            }
            let pascal = capitalize(&camel);
            if bindings.bindings.contains_key(pascal.as_str()) {
                return Some(pascal);
            }
            None
        };

        if let Some((base, suffix)) = component.split_once('.') {
            let resolved_base = resolve_base(base)?;
            return Some(cstr!("_ctx.{}.{}", resolved_base, suffix));
        }
        resolve_base(component).map(|binding| cstr!("_ctx.{}", binding))
    }

    pub(crate) fn is_self_component_reference(&self, component: &str) -> bool {
        self.experimental_self_component
            && component == "Self"
            && self.component_name.is_some_and(|name| !name.is_empty())
    }

    pub(crate) fn component_resolution_name(&self, component: &str) -> String {
        if self.is_self_component_reference(component)
            && let Some(component_name) = self.component_name
        {
            component_name.to_compact_string()
        } else {
            component.to_compact_string()
        }
    }
}
