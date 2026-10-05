//! Authored defaults retained before imported prop declarations are expanded.

use super::{MacroTracker, PropDefinition};
use vize_carton::{CompactString, FxHashMap, FxHashSet};

#[derive(Clone, Default)]
pub(crate) struct StaticDefaultObject {
    pub values: FxHashMap<CompactString, CompactString>,
    pub removed: FxHashSet<CompactString>,
    pub clears_previous: bool,
}

impl StaticDefaultObject {
    pub fn clear(&mut self) {
        self.values.clear();
        self.removed.clear();
        self.clears_previous = true;
    }

    pub fn spread(&mut self, other: Self) {
        if other.clears_previous {
            self.clear();
        }
        for name in other.removed {
            self.values.remove(&name);
            self.removed.insert(name);
        }
        for (name, value) in other.values {
            self.removed.remove(&name);
            self.values.insert(name, value);
        }
    }
}

#[derive(Default)]
pub(super) struct WithDefaults {
    expression: Option<CompactString>,
    values: FxHashMap<CompactString, CompactString>,
    resolved_names: FxHashSet<CompactString>,
    objects: FxHashMap<CompactString, StaticDefaultObject>,
}

impl MacroTracker {
    /// Non-undefined defaults proven by the existing imported-object resolver.
    /// Keep these facts separate from the exact authored default expressions.
    pub fn set_resolved_with_defaults(&mut self, names: FxHashSet<CompactString>) {
        self.with_defaults.resolved_names = names;
    }

    /// Names of defaults whose non-undefined values were resolved externally.
    pub fn resolved_with_defaults(&self) -> impl Iterator<Item = &str> {
        self.with_defaults
            .resolved_names
            .iter()
            .map(CompactString::as_str)
    }

    /// The exact authored second argument of `withDefaults`, including opaque
    /// expressions for which individual default values cannot be proven.
    pub fn with_defaults_expression(&self) -> Option<&str> {
        self.with_defaults.expression.as_deref()
    }

    pub(crate) fn set_with_defaults(
        &mut self,
        expression: CompactString,
        values: FxHashMap<CompactString, CompactString>,
    ) {
        self.with_defaults.expression = Some(expression);
        self.with_defaults.values = values;
        for prop in &mut self.props {
            if let Some(value) = self.with_defaults.values.get(&prop.name) {
                prop.default_value = Some(value.clone());
            }
        }
    }

    pub(super) fn apply_with_default(&self, prop: &mut PropDefinition) {
        if let Some(value) = self.with_defaults.values.get(&prop.name) {
            prop.default_value = Some(value.clone());
        }
    }

    pub(crate) fn default_object(&self, name: &str) -> Option<&StaticDefaultObject> {
        self.with_defaults.objects.get(name)
    }

    pub(crate) fn record_default_object(
        &mut self,
        name: &str,
        object: Option<StaticDefaultObject>,
    ) {
        self.with_defaults.objects.remove(name);
        if let Some(object) = object {
            self.with_defaults
                .objects
                .insert(CompactString::new(name), object);
        }
    }

    pub(crate) fn invalidate_default_objects(&mut self) {
        self.with_defaults.objects.clear();
    }
}
