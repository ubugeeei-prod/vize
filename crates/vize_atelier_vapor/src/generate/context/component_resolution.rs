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

    pub(crate) fn resolve_directive_binding_expr(&self, directive: &str) -> Option<String> {
        let bindings = self.binding_metadata?;
        if !bindings.is_script_setup {
            return None;
        }
        let name = cstr!("v{}", capitalize(&camelize(directive)));
        bindings
            .bindings
            .contains_key(name.as_str())
            .then(|| cstr!("_ctx.{name}"))
    }

    pub(crate) fn is_self_component_reference(&self, component: &str) -> bool {
        self.component_name.is_some_and(|name| {
            !name.is_empty()
                && (self.experimental_self_component && component == "Self"
                    || component == name
                    || capitalize(&camelize(component)) == name)
        })
    }

    pub(crate) fn component_resolution_name(&self, component: &str) -> String {
        if self.experimental_self_component
            && component == "Self"
            && let Some(component_name) = self.component_name
            && !component_name.is_empty()
        {
            component_name.to_compact_string()
        } else {
            component.to_compact_string()
        }
    }
}
