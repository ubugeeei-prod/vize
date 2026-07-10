use crate::options::BindingType;
use crate::{CodegenContext, RuntimeHelper};
use vize_carton::{String, ToCompactString, camelize, capitalize};

struct ComponentBinding {
    name: String,
    binding_type: BindingType,
    suffix: Option<String>,
}

impl CodegenContext {
    /// Check if a component is in binding metadata (from script setup).
    pub fn is_component_in_bindings(&self, component: &str) -> bool {
        self.resolve_component_binding_name(component).is_some()
    }

    /// Resolve the binding name for a component tag.
    pub fn resolve_component_binding_name(&self, component: &str) -> Option<String> {
        self.resolve_component_binding(component).map(|binding| {
            if let Some(suffix) = binding.suffix {
                let mut name = String::with_capacity(binding.name.len() + suffix.len() + 1);
                name.push_str(binding.name.as_str());
                name.push('.');
                name.push_str(suffix.as_str());
                name
            } else {
                binding.name
            }
        })
    }

    /// Push a component tag that resolves to a setup binding.
    pub fn push_component_binding_tag(&mut self, component: &str) -> bool {
        let Some(binding) = self.resolve_component_binding(component) else {
            return false;
        };

        let needs_unref = self.options.inline
            && matches!(
                binding.binding_type,
                BindingType::SetupLet | BindingType::SetupMaybeRef | BindingType::SetupRef
            );
        if needs_unref {
            self.use_helper(RuntimeHelper::Unref);
            self.push(self.helper(RuntimeHelper::Unref));
            self.push("(");
        }
        if !self.options.inline {
            self.push("$setup.");
        }
        self.push(binding.name.as_str());
        if needs_unref {
            self.push(")");
        }
        if let Some(suffix) = binding.suffix {
            self.push(".");
            self.push(suffix.as_str());
        }
        true
    }

    fn resolve_component_binding(&self, component: &str) -> Option<ComponentBinding> {
        let metadata = self.options.binding_metadata.as_ref()?;

        let resolve_base = |name: &str| {
            if let Some(binding_type) = metadata.bindings.get(name) {
                return Some((name.to_compact_string(), *binding_type));
            }

            let camel = camelize(name);
            if let Some(binding_type) = metadata.bindings.get(camel.as_str()) {
                return Some((camel, *binding_type));
            }

            let pascal = capitalize(&camel);
            if let Some(binding_type) = metadata.bindings.get(pascal.as_str()) {
                return Some((pascal, *binding_type));
            }

            None
        };

        if let Some((base, suffix)) = component.split_once('.') {
            let (name, binding_type) = resolve_base(base)?;
            return Some(ComponentBinding {
                name,
                binding_type,
                suffix: Some(suffix.to_compact_string()),
            });
        }

        let (name, binding_type) = resolve_base(component)?;
        Some(ComponentBinding {
            name,
            binding_type,
            suffix: None,
        })
    }
}
