mod no_inline_template;
mod no_suspense;
mod prefer_static_class;
mod require_vapor_attribute;

use crate::rule::RuleRegistry;

pub use no_inline_template::NoInlineTemplate;
pub use no_suspense::NoSuspense;
pub use prefer_static_class::PreferStaticClass;
pub use require_vapor_attribute::RequireVaporAttribute;

pub(crate) fn register(registry: &mut RuleRegistry) {
    registry.register(Box::new(NoSuspense));
    registry.register(Box::new(NoInlineTemplate));
    registry.register(Box::new(PreferStaticClass));
    registry.register(Box::new(RequireVaporAttribute::default()));
}
