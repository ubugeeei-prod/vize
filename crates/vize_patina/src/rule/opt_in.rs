use super::RuleRegistry;

pub(super) fn register(registry: &mut RuleRegistry) {
    #[cfg(not(target_arch = "wasm32"))]
    registry.register(Box::new(crate::rules::type_aware::StrictBooleanExpressions));
    crate::rules::a11y::register_opt_in(registry);
    crate::rules::ecosystem::register_opt_in(registry);
    crate::rules::petite_vue::register_opt_in(registry);
    crate::rules::vue::register_opt_in(registry);
    crate::rules::facts::register_opt_in(registry);
}
