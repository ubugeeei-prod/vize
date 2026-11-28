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
        if !self.jsx_closure && binding.binding_type == BindingType::Props {
            return Some(self.prop_component_tag(binding));
        }
        let needs_unref = !self.jsx_closure
            && self.options.inline
            && matches!(
                binding.binding_type,
                BindingType::SetupLet | BindingType::SetupMaybeRef | BindingType::SetupRef
            );
        let mut resolved = String::default();

        if needs_unref {
            self.use_core_helper(RuntimeHelper::Unref);
            resolved.push_str("_unref(");
        } else if !self.options.inline && !self.jsx_closure {
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

    /// A prop used as a tag reads the props object (`__props` inline,
    /// `$props` otherwise) through `_unref`, as Vue does. Out of line: an
    /// ordinary tag never takes this path.
    #[cold]
    #[inline(never)]
    fn prop_component_tag(&mut self, binding: ComponentBinding) -> String {
        self.use_core_helper(RuntimeHelper::Unref);
        let mut resolved = String::from(if self.options.inline {
            "_unref(__props[\""
        } else {
            "_unref($props[\""
        });
        resolved.push_str(&super::element::props::escape_js_string(
            binding.name.as_str(),
        ));
        resolved.push_str("\"])");
        if let Some(suffix) = binding.suffix {
            resolved.push('.');
            resolved.push_str(suffix.as_str());
        }
        resolved
    }

    fn resolve_component_binding(&self, component: &str) -> Option<ComponentBinding> {
        if self.experimental_self_component && component == "Self" {
            return None;
        }

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
        let (mut name, mut binding_type) = resolve_base(base)?;
        if matches!(binding_type, BindingType::Props | BindingType::PropsAliased) {
            (name, binding_type) = resolve_props_spelling(metadata, base)?;
        }
        Some(ComponentBinding {
            name,
            binding_type,
            suffix: suffix.map(|suffix| suffix.to_compact_string()),
        })
    }
}

/// Vue's `resolveSetupReference` once a spelling of the tag names a prop: a
/// setup binding under any spelling wins, then a prop. A renamed props
/// destructure (`{ icon: Icon }`) is not a setup reference, so its tag
/// resolves as an asset. Out of line: an ordinary tag never takes this path.
#[cold]
#[inline(never)]
fn resolve_props_spelling(
    metadata: &vize_atelier_core::options::BindingMetadata,
    name: &str,
) -> Option<(String, BindingType)> {
    let camel = camelize(name);
    let pascal = capitalize(camel.as_str());
    let spellings = [name, camel.as_str(), pascal.as_str()];
    let kind = |spelling: &str| metadata.bindings.get(spelling).copied();
    spellings
        .iter()
        .find_map(|spelling| {
            kind(spelling)
                .filter(|kind| !matches!(kind, BindingType::Props | BindingType::PropsAliased))
                .map(|kind| (spelling.to_compact_string(), kind))
        })
        .or_else(|| {
            spellings
                .iter()
                .find(|spelling| kind(spelling) == Some(BindingType::Props))
                .map(|spelling| (spelling.to_compact_string(), BindingType::Props))
        })
}
