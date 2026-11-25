use crate::options::BindingType;
use crate::{CodegenContext, ElementNode, RuntimeHelper};

use super::document::EmitDocument;
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
        // Non-inline setup components are bracket reads (`$setup["MyButton"]`).
        // Expression identifiers stay `$setup.msg`.
        if self.options.inline {
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

    /// Anchor the setup binding [`Self::push_component_binding_tag`] just
    /// wrote for `el` at its authored tag name. A resolved asset is anchored
    /// where it is resolved, but a binding is read in place. Callers make the
    /// one recording check after a binding was written, so a tag that is not
    /// one, and every compile without a map, does what it did before.
    #[cold]
    #[inline(never)]
    pub(in crate::codegen) fn anchor_component_binding(&mut self, el: &ElementNode<'_>) {
        // Measure the read by writing it again into a scratch document. The
        // only other effect, using the `unref` helper, is already recorded.
        let written = std::mem::replace(&mut self.out, EmitDocument::new(false));
        self.push_component_binding_tag(el.tag);
        let read = std::mem::replace(&mut self.out, written).len() as u32;
        self.out.anchor_written(read, el.loc.span.start + 1);
    }

    fn resolve_component_binding(&self, component: &str) -> Option<ComponentBinding> {
        if self.experimental_self_component && component == "Self" {
            return None;
        }

        let metadata = self.options.binding_metadata.as_ref()?;

        let resolve_base = |name: &str| {
            let mut prop_fallback = None;
            let mut resolve_candidate = |candidate: String| {
                if let Some(binding_type) = metadata.bindings.get(candidate.as_str()) {
                    if matches!(binding_type, BindingType::Props | BindingType::PropsAliased) {
                        prop_fallback.get_or_insert((candidate, *binding_type));
                    } else {
                        return Some((candidate, *binding_type));
                    }
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
