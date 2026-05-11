mod no_floating_promises;
mod no_reactivity_loss;
mod no_unsafe_template_binding;

use crate::rule::RuleRegistry;

pub use no_floating_promises::NoFloatingPromises;
pub use no_reactivity_loss::NoReactivityLoss;
pub use no_unsafe_template_binding::NoUnsafeTemplateBinding;

pub(crate) fn register(registry: &mut RuleRegistry) {
    registry.register(Box::new(NoFloatingPromises));
    registry.register(Box::new(NoReactivityLoss));
    registry.register(Box::new(NoUnsafeTemplateBinding));
}
