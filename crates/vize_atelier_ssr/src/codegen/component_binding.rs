use super::SsrCodegenContext;
use vize_atelier_core::{BindingType, RuntimeHelper};
use vize_l0::{String, ToCompactString, camelize, capitalize};

struct ComponentBinding {
    name: String,
    binding_type: BindingType,
    suffix: Option<String>,
}

impl<'a> SsrCodegenContext<'a> {
    pub(crate) fn resolve_component_binding_expr(&mut self, component: &str) -> Option<String> {
        let binding = self.resolve_component_binding(component)?;
        // A prop is read off the props object (`__props` inline, `$props`
        // otherwise), through `_unref` as Vue does.
        let is_prop = !self.jsx_closure && binding.binding_type == BindingType::Props;
        let needs_unref = is_prop
            || (!self.jsx_closure
                && self.options.inline
                && matches!(
                    binding.binding_type,
                    BindingType::SetupLet | BindingType::SetupMaybeRef | BindingType::SetupRef
                ));
        let mut resolved = String::default();

        if needs_unref {
            self.use_core_helper(RuntimeHelper::Unref);
            resolved.push_str("_unref(");
        }
        if is_prop {
            resolved.push_str(if self.options.inline {
                "__props[\""
            } else {
                "$props[\""
            });
            resolved.push_str(&super::element::props::escape_js_string(
                binding.name.as_str(),
            ));
            resolved.push_str("\"]");
        } else {
            if !needs_unref && !self.options.inline && !self.jsx_closure {
                resolved.push_str("$setup.");
            }
            resolved.push_str(binding.name.as_str());
        }
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
        if self.experimental_self_component && component == "Self" {
            return None;
        }

        let metadata = self.options.binding_metadata.as_ref()?;
        // Vue's `resolveSetupReference`: setup bindings first, then a prop.
        // A renamed props destructure (`{ icon: Icon }`) is not a setup
        // reference, so its tag resolves as an asset.
        let resolve_base = |name: &str| {
            let mut prop_fallback = None;
            let mut candidate = |candidate: &str| match metadata.bindings.get(candidate) {
                Some(BindingType::Props) => {
                    if prop_fallback.is_none() {
                        prop_fallback = Some((candidate.to_compact_string(), BindingType::Props));
                    }
                    None
                }
                Some(BindingType::PropsAliased) | None => None,
                Some(binding_type) => Some((candidate.to_compact_string(), *binding_type)),
            };
            if let Some(binding) = candidate(name) {
                return Some(binding);
            }
            let camel = camelize(name);
            if let Some(binding) = candidate(camel.as_str()) {
                return Some(binding);
            }
            if let Some(binding) = candidate(capitalize(camel.as_str()).as_str()) {
                return Some(binding);
            }
            prop_fallback
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
