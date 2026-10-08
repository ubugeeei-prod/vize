//! Authored registration names, separate from resolved component identity.

use vize_carton::{CompactString, FxHashSet};

#[derive(Debug, Default)]
pub struct TemplateComponentRegistrations {
    pub(crate) module_names: FxHashSet<CompactString>,
    pub(crate) option_names: FxHashSet<CompactString>,
}

impl TemplateComponentRegistrations {
    /// Whether an exact PascalCase name is registered in this component.
    pub fn contains(&self, name: &str, script_setup: bool) -> bool {
        self.option_names.contains(name) || (script_setup && self.module_names.contains(name))
    }

    pub(crate) fn extend(&mut self, other: Self) {
        self.module_names.extend(other.module_names);
        self.option_names.extend(other.option_names);
    }
}
