//! Property receivers for template names which cannot be declared as locals.
//!
//! Props and Options API members have different owners. Keep that ownership in
//! one projection input so guards, loops, event handlers and prop checks resolve
//! reserved names through the same typed object.

use super::{helpers::is_reserved_identifier, props::collect_template_prop_names};
use vize_carton::{FxHashMap, FxHashSet, String};
use vize_croquis::{BindingType, Croquis};

#[derive(Clone, Copy, Debug)]
enum Receiver {
    Props,
    OptionsInstance,
}

#[derive(Default, Debug)]
pub(crate) struct TemplateBindingAccess(FxHashMap<String, Receiver>);

impl TemplateBindingAccess {
    pub(crate) fn collect(summary: &Croquis, options_api: bool) -> Self {
        let mut result = Self::from_props(collect_template_prop_names(summary));
        if options_api {
            for (name, binding) in &summary.bindings.bindings {
                if (matches!(binding, BindingType::Data | BindingType::Options)
                    || (*binding == BindingType::Props && !summary.bindings.is_script_setup))
                    && is_reserved_identifier(name)
                {
                    result
                        .0
                        .insert(name.as_str().into(), Receiver::OptionsInstance);
                }
            }
        }
        result
    }

    pub(crate) fn from_props(names: FxHashSet<String>) -> Self {
        Self(
            names
                .into_iter()
                .map(|name| (name, Receiver::Props))
                .collect(),
        )
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    pub(crate) fn receiver(&self, name: &str) -> Option<&'static str> {
        self.0.get(name).map(|receiver| match receiver {
            Receiver::Props => "props",
            Receiver::OptionsInstance => "__vize_options_instance",
        })
    }
}
