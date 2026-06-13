mod no_duplicate_class;
use no_duplicate_class::NoDuplicateClass;

use crate::rule::RuleRegistry;

pub(crate) fn register(registry: &mut RuleRegistry) {
    registry.register(Box::new(NoDuplicateClass));
}
