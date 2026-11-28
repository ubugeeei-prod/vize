use vize_atelier_core::options::{BindingMetadata, BindingType};
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
            if is_props_binding(bindings, &resolved_base) {
                return self.resolve_props_component_expr(bindings, base, Some(suffix));
            }
            return Some(if self.jsx_closure {
                cstr!("{}.{}", resolved_base, suffix)
            } else {
                cstr!("_ctx.{}.{}", resolved_base, suffix)
            });
        }
        let binding = resolve_base(component)?;
        if is_props_binding(bindings, &binding) {
            return self.resolve_props_component_expr(bindings, component, None);
        }
        Some(if self.jsx_closure {
            binding
        } else {
            cstr!("_ctx.{}", binding)
        })
    }

    /// Vue's `resolveSetupReference` once a spelling of the tag names a prop:
    /// a setup binding under any spelling wins, then a prop, read off the
    /// render function's `$props`. A renamed props destructure
    /// (`{ icon: Icon }`) is not a setup reference, so its tag resolves as an
    /// asset. Out of line: an ordinary tag never takes this path.
    #[cold]
    #[inline(never)]
    fn resolve_props_component_expr(
        &self,
        bindings: &BindingMetadata,
        name: &str,
        suffix: Option<&str>,
    ) -> Option<String> {
        let camel = camelize(name);
        let pascal = capitalize(&camel);
        let spellings = [name, camel.as_str(), pascal.as_str()];
        let kind = |spelling: &str| bindings.bindings.get(spelling).copied();
        let (binding, owner) = spellings
            .iter()
            .find(|spelling| kind(spelling).is_some_and(|kind| !is_props_kind(kind)))
            .map(|spelling| (*spelling, "_ctx."))
            .or_else(|| {
                spellings
                    .iter()
                    .find(|spelling| kind(spelling) == Some(BindingType::Props))
                    .map(|spelling| (*spelling, "$props."))
            })?;
        let owner = if self.jsx_closure { "" } else { owner };
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

fn is_props_kind(kind: BindingType) -> bool {
    matches!(kind, BindingType::Props | BindingType::PropsAliased)
}

fn is_props_binding(bindings: &BindingMetadata, name: &str) -> bool {
    bindings
        .bindings
        .get(name)
        .is_some_and(|kind| is_props_kind(*kind))
}
