use super::RuleRegistry;

pub(super) fn register(registry: &mut RuleRegistry) {
    #[cfg(not(target_arch = "wasm32"))]
    if !registry.has_rule("type/strict-boolean-expressions") {
        registry.register(Box::new(crate::rules::type_aware::StrictBooleanExpressions));
    }
    crate::rules::a11y::register_opt_in(registry);
    crate::rules::ecosystem::register_opt_in(registry);
    crate::rules::petite_vue::register_opt_in(registry);
    crate::rules::vue::register_opt_in(registry);
    crate::rules::facts::register_opt_in(registry);
}

#[cfg(test)]
mod tests {
    use super::RuleRegistry;

    #[test]
    fn repeated_opt_in_registration_preserves_all_names_and_order() {
        let mut registry = RuleRegistry::with_opt_in_rules();
        let expected = registry.rule_names().to_vec();
        for _ in 0..2 {
            registry.register_opt_in_rules();
            assert_eq!(registry.rule_names(), expected.as_slice());
            assert_eq!(registry.rules().len(), expected.len());
        }
    }
}
