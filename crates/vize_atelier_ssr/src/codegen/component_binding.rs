use super::SsrCodegenContext;
use vize_atelier_core::{BindingType, RuntimeHelper};
use vize_carton::{String, ToCompactString, camelize, capitalize};

struct ComponentBinding {
    name: String,
    binding_type: BindingType,
    suffix: Option<String>,
}

impl<'a> SsrCodegenContext<'a> {
    pub(crate) fn resolve_component_binding_expr(&mut self, component: &str) -> Option<String> {
        let binding = self.resolve_component_binding(component)?;
        let needs_unref = self.options.inline
            && matches!(
                binding.binding_type,
                BindingType::SetupLet | BindingType::SetupMaybeRef | BindingType::SetupRef
            );
        let mut resolved = String::default();

        if needs_unref {
            self.use_core_helper(RuntimeHelper::Unref);
            resolved.push_str("_unref(");
        } else if !self.options.inline {
            resolved.push_str("$setup.");
        }
        resolved.push_str(binding.name.as_str());
        if needs_unref {
            resolved.push(')');
        }
        if let Some(suffix) = binding.suffix {
            resolved.push('.');
            resolved.push_str(suffix.as_str());
        }

        Some(resolved)
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
            let pascal = capitalize(camel.as_str());
            metadata
                .bindings
                .get(pascal.as_str())
                .map(|binding_type| (pascal, *binding_type))
        };

        let (base, suffix) = component
            .split_once('.')
            .map_or((component, None), |(base, suffix)| (base, Some(suffix)));
        let (name, binding_type) = resolve_base(base)?;
        Some(ComponentBinding {
            name,
            binding_type,
            suffix: suffix.map(|suffix| suffix.to_compact_string()),
        })
    }
}
