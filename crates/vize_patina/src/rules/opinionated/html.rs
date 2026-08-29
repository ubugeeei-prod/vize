mod no_dupe_style_properties;
mod no_duplicate_class;

use crate::rule::RuleRegistry;
pub(crate) use no_dupe_style_properties::NoDupeStyleProperties;
pub(crate) use no_duplicate_class::NoDuplicateClass;

pub(crate) fn register(registry: &mut RuleRegistry) {
    registry.register(Box::new(NoDuplicateClass));
    registry.register(Box::new(NoDupeStyleProperties));
}
