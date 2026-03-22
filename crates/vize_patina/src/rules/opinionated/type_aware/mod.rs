mod no_floating_promises;

use crate::rule::RuleRegistry;

pub use no_floating_promises::NoFloatingPromises;

pub(crate) fn register(registry: &mut RuleRegistry) {
    registry.register(Box::new(NoFloatingPromises::default()));
}
