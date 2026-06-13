mod no_dupe_style_properties;
mod no_duplicate_class;

use crate::rule::RuleRegistry;
use no_dupe_style_properties::NoDupeStyleProperties;
use no_duplicate_class::NoDuplicateClass;

pub(crate) fn register(registry: &mut RuleRegistry) {
    registry.register(Box::new(NoDuplicateClass));
    registry.register(Box::new(NoDupeStyleProperties));
}
