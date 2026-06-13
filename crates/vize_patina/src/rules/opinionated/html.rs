mod no_dupe_style_properties;

use crate::rule::RuleRegistry;
use no_dupe_style_properties::NoDupeStyleProperties;

pub(crate) fn register(registry: &mut RuleRegistry) {
    registry.register(Box::new(NoDupeStyleProperties));
}
