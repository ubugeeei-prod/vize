use super::*;
use vize_patina::rules::a11y::{NoAccessKey, NoAutofocus};

#[test]
fn genuine_focus_builtins_still_have_no_bare_element_capability_or_capture_admission() {
    for rule in [
        Box::new(NoAutofocus) as Box<dyn Rule>,
        Box::new(NoAccessKey) as Box<dyn Rule>,
    ] {
        let name = rule.meta().name;
        assert!(rule.as_native_template_rule().is_none());
        let log = events();
        let audit = Audit::new(&FIRST, "provided", Profile::Bindings, log.clone());
        let calls = audit.profile_calls.clone();
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(audit));
        registry.register(rule);
        let configured = Linter::with_registry(registry);
        assert_eq!(
            configured
                .lint_native_template("<div autofocus accesskey='h'/>", "test.vue")
                .unwrap_err(),
            Refusal::UnprovidedRule { rule: name.into() }
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(log.lock().unwrap().is_empty());
    }
}
