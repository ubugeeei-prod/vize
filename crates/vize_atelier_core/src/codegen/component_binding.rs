use crate::options::BindingType;
use crate::{CodegenContext, RuntimeHelper};
use vize_l0::{String, ToCompactString, camelize, capitalize};

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

        // A prop is read off the props object (`__props` in the inlined
        // render function, `$props` otherwise), through `_unref` as Vue does.
        let is_prop = binding.binding_type == BindingType::Props;
        let needs_unref = is_prop
            || (self.options.inline
                && matches!(
                    binding.binding_type,
                    BindingType::SetupLet | BindingType::SetupMaybeRef | BindingType::SetupRef
                ));
        if needs_unref {
            self.use_helper(RuntimeHelper::Unref);
            self.push(self.helper(RuntimeHelper::Unref));
            self.push("(");
        }
        // Non-inline setup components are bracket reads (`$setup["MyButton"]`).
        // Expression identifiers stay `$setup.msg`.
        if is_prop {
            self.push(if self.options.inline {
                "__props[\""
            } else {
                "$props[\""
            });
            let escaped = super::helpers::escape_js_string(binding.name.as_str());
            self.push(escaped.as_str());
            self.push("\"]");
        } else if self.options.inline {
            self.push(binding.name.as_str());
        } else {
            self.push("$setup[\"");
            let escaped = super::helpers::escape_js_string(binding.name.as_str());
            self.push(escaped.as_str());
            self.push("\"]");
        }
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
        if self.experimental_self_component && component == "Self" {
            return None;
        }

        let metadata = self.options.binding_metadata.as_ref()?;

        let resolve_base = |name: &str| {
            let mut prop_fallback = None;
            // Vue's `resolveSetupReference`: setup bindings first, then a
            // prop. A renamed props destructure (`{ icon: Icon }`) is not a
            // setup reference, so its tag resolves as an asset.
            let mut resolve_candidate = |candidate: String| {
                match metadata.bindings.get(candidate.as_str()) {
                    Some(BindingType::Props) => {
                        prop_fallback.get_or_insert((candidate, BindingType::Props));
                    }
                    Some(BindingType::PropsAliased) | None => {}
                    Some(binding_type) => return Some((candidate, *binding_type)),
                }
                None
            };

            if let Some(binding) = resolve_candidate(name.to_compact_string()) {
                return Some(binding);
            }
            let camel = camelize(name);
            if camel.as_str() != name
                && let Some(binding) = resolve_candidate(camel.clone())
            {
                return Some(binding);
            }

            let pascal = capitalize(&camel);
            if pascal.as_str() != name
                && pascal.as_str() != camel.as_str()
                && let Some(binding) = resolve_candidate(pascal)
            {
                return Some(binding);
            }

            prop_fallback
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
