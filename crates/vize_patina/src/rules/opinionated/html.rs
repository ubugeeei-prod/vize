mod no_duplicate_class;
use no_duplicate_class::NoDuplicateClass;

use crate::rule::RuleRegistry;

pub(crate) fn register(registry: &mut RuleRegistry) {
    registry.register(Box::new(NoDuplicateClass));
mod no_dupe_style_properties;

use crate::rule::RuleRegistry;
use no_dupe_style_properties::NoDupeStyleProperties;

pub(crate) fn register(registry: &mut RuleRegistry) {
    registry.register(Box::new(NoDupeStyleProperties));
}
