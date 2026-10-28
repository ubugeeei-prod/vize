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
pub(crate) struct TemplateBindingAccess {
    receivers: FxHashMap<String, Receiver>,
    lexical: FxHashSet<String>,
}

impl TemplateBindingAccess {
    pub(crate) fn collect(summary: &Croquis, options_api: bool) -> Self {
        let mut result = Self::from_props(collect_template_prop_names(summary));
        if options_api {
            super::script_facts::with_bindings(summary, |bindings| {
                let script_setup = bindings.is_script_setup();
                for (name, binding) in bindings.typed() {
                    if matches!(binding, BindingType::Data | BindingType::Options)
                        || (binding == BindingType::Props && !script_setup)
                    {
                        if is_reserved_identifier(name) {
                            result.lexical.remove(name);
                            result
                                .receivers
                                .insert(name.into(), Receiver::OptionsInstance);
                        } else {
                            result.receivers.remove(name);
                            result.lexical.insert(name.into());
                        }
                    }
                }
            });
        }
        result
    }

    pub(crate) fn from_props(names: FxHashSet<String>) -> Self {
        Self {
            receivers: names
                .into_iter()
                .map(|name| (name, Receiver::Props))
                .collect(),
            lexical: FxHashSet::default(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.receivers.is_empty()
    }

    pub(crate) fn contains(&self, name: &str) -> bool {
        self.receivers.contains_key(name)
    }

    pub(crate) fn is_known(&self, name: &str) -> bool {
        self.contains(name) || self.lexical.contains(name)
    }

    pub(crate) fn receiver(&self, name: &str) -> Option<&'static str> {
        self.receivers.get(name).map(|receiver| match receiver {
            Receiver::Props => "props",
            Receiver::OptionsInstance => "__vize_options_instance",
        })
    }
}
