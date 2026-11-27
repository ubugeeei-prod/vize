use vize_atelier_core::options::{BindingMetadata, BindingType};
use vize_carton::{String, ToCompactString, camelize, capitalize, cstr};

use super::GenerateContext;

impl<'a> GenerateContext<'a> {
    pub(crate) fn resolve_component_binding_expr(&self, component: &str) -> Option<String> {
        if self.experimental_self_component && component == "Self" {
            return None;
        }

        let bindings = self.binding_metadata?;
        let (base, suffix) = match component.split_once('.') {
            Some((base, suffix)) => (base, Some(suffix)),
            None => (component, None),
        };
        let (binding, is_prop) = resolve_setup_reference(bindings, base)?;
        // A prop is read off the render function's `$props`, as Vue does.
        let owner = if self.jsx_closure {
            ""
        } else if is_prop {
            "$props."
        } else {
            "_ctx."
        };
        Some(match suffix {
            Some(suffix) => cstr!("{owner}{binding}.{suffix}"),
            None => cstr!("{owner}{binding}"),
        })
    }

    pub(crate) fn resolve_directive_binding_expr(&self, directive: &str) -> Option<String> {
        let bindings = self.binding_metadata?;
        if !bindings.is_script_setup {
            return None;
        }
        let name = cstr!("v{}", capitalize(&camelize(directive)));
        bindings.bindings.contains_key(name.as_str()).then(|| {
            if self.jsx_closure {
                name
            } else {
                cstr!("_ctx.{name}")
            }
        })
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

/// The binding a component tag names, and whether it is a prop: Vue's
/// `resolveSetupReference` tries the tag, its camelized and its PascalCased
/// spelling for a setup binding first and for a prop last. A renamed props
/// destructure (`{ icon: Icon }`) is not a setup reference, so its tag
/// resolves as an asset.
pub(crate) fn resolve_setup_reference(
    bindings: &BindingMetadata,
    name: &str,
) -> Option<(String, bool)> {
    let mut prop = None;
    let mut candidate = |candidate: &str| match bindings.bindings.get(candidate) {
        Some(BindingType::Props) => {
            if prop.is_none() {
                prop = Some(candidate.to_compact_string());
            }
            None
        }
        Some(BindingType::PropsAliased) | None => None,
        Some(_) => Some(candidate.to_compact_string()),
    };
    if let Some(binding) = candidate(name) {
        return Some((binding, false));
    }
    let camel = camelize(name);
    if camel.as_str() != name
        && let Some(binding) = candidate(camel.as_str())
    {
        return Some((binding, false));
    }
    let pascal = capitalize(&camel);
    if pascal.as_str() != name
        && pascal.as_str() != camel.as_str()
        && let Some(binding) = candidate(pascal.as_str())
    {
        return Some((binding, false));
    }
    prop.map(|binding| (binding, true))
}
