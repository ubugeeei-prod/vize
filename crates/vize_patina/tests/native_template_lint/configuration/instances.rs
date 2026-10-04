use super::super::support::{SOURCE, complete};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use vize_l0::config::VueVersion;
use vize_l1::markup::NativeLintComponent;
use vize_patina::{
    LintContext, LintPreset, Linter, Rule, RuleCategory, RuleMeta, RuleRegistry, Severity,
    native::template::{
        NativeTemplateLintContext, NativeTemplateLintRefusal as Refusal, NativeTemplateRule,
    },
};
use vize_relief::RootNode;

static AUDIT_META: RuleMeta = RuleMeta {
    name: "test/native-configured-root",
    description: "Observe the actual configured instance",
    category: RuleCategory::StronglyRecommended,
    fixable: false,
    default_severity: Severity::Warning,
};

struct AuditRule {
    version: Option<VueVersion>,
    vapor: Option<bool>,
    marker: &'static str,
    called: Arc<AtomicUsize>,
}
impl Rule for AuditRule {
    fn meta(&self) -> &'static RuleMeta {
        &AUDIT_META
    }
    fn as_native_template_rule(&self) -> Option<&dyn NativeTemplateRule> {
        Some(self)
    }
    fn run_on_template<'a>(&self, context: &mut LintContext<'a>, root: &RootNode<'a>) {
        context.warn_with_help(
            context.t_fmt(
                "vue/component-definition-name-casing.message",
                &[("name", self.marker)],
            ),
            &root.loc,
            context.t("vue/component-definition-name-casing.help"),
        );
    }
}
impl NativeTemplateRule for AuditRule {
    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        root: &NativeLintComponent<'a>,
    ) -> Result<(), Refusal> {
        assert!(core::ptr::eq(context.owner(), root));
        assert!(core::ptr::eq(
            context.source(),
            root.component().block().source()
        ));
        assert!(core::ptr::eq(
            context.source(),
            root.component().block().root_source()
        ));
        assert_eq!(root.component().block().start(), 0);
        assert_eq!(context.requested_vue_version(), self.version);
        assert_eq!(context.requested_vapor_mode(), self.vapor);
        self.called.fetch_add(1, Ordering::SeqCst);
        context.warn_root_with_help(
            "vue/component-definition-name-casing.message",
            &[("name", self.marker)],
            "vue/component-definition-name-casing.help",
        );
        Ok(())
    }
}

#[test]
fn all_four_original_constructors_preserve_unspecified_host_options() {
    for base in [
        Linter::new(),
        Linter::with_preset(LintPreset::Incremental),
        Linter::with_ecosystem(),
        Linter::with_registry(RuleRegistry::new()),
    ] {
        let called = Arc::new(AtomicUsize::new(0));
        let linter = base
            .with_enabled_rules(Some(vec![AUDIT_META.name.into()]))
            .with_rule(Box::new(AuditRule {
                version: None,
                vapor: None,
                marker: "actual",
                called: called.clone(),
            }));
        let original = linter.lint_template(SOURCE, "unchanged.vue");
        let native = linter
            .lint_native_template(SOURCE, "unchanged.vue")
            .unwrap();
        assert_eq!(complete(&native), complete(&original));
        assert_eq!(native.diagnostics[0].rule_name, AUDIT_META.name);
        assert_eq!(called.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn explicit_and_unspecified_vue3_vapor_requests_reach_the_actual_callback() {
    for version in [None, Some(VueVersion::V3)] {
        for vapor in [None, Some(false)] {
            let called = Arc::new(AtomicUsize::new(0));
            let mut registry = RuleRegistry::new();
            registry.register(Box::new(AuditRule {
                version,
                vapor,
                marker: "request",
                called: called.clone(),
            }));
            let linter = Linter::with_registry(registry)
                .with_vue_version(version)
                .with_vapor_mode(vapor);
            assert_eq!(
                complete(&linter.lint_native_template(SOURCE, "path.vue").unwrap()),
                complete(&linter.lint_template(SOURCE, "path.vue"))
            );
            assert_eq!(called.load(Ordering::SeqCst), 1);
        }
    }
}

#[test]
fn resetting_authored_compatibility_options_keeps_the_existing_sticky_mutations() {
    let called = Arc::new(AtomicUsize::new(0));
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(AuditRule {
        version: None,
        vapor: None,
        marker: "reset",
        called: called.clone(),
    }));
    let linter = Linter::with_registry(registry)
        .with_vue_version(Some(VueVersion::V2))
        .with_vue_version(None)
        .with_vapor_mode(Some(false))
        .with_vapor_mode(None);
    for name in [
        "vue/prefer-props-shorthand",
        "vue/no-deprecated-slot-attribute",
        "script/no-get-current-instance",
        "script/no-next-tick",
    ] {
        assert!(!linter.is_rule_enabled(name));
    }
    assert_eq!(
        complete(&linter.lint_native_template(SOURCE, "reset.vue").unwrap()),
        complete(&linter.lint_template(SOURCE, "reset.vue"))
    );
    assert_eq!(called.load(Ordering::SeqCst), 1);
}

#[test]
fn actual_duplicate_registry_instances_preserve_callback_and_diagnostic_order() {
    let called = Arc::new(AtomicUsize::new(0));
    let mut registry = RuleRegistry::new();
    for marker in ["first", "second"] {
        registry.register(Box::new(AuditRule {
            version: None,
            vapor: None,
            marker,
            called: called.clone(),
        }));
    }
    let linter = Linter::with_registry(registry);
    let native = linter.lint_native_template(SOURCE, "order.vue").unwrap();
    let original = linter.lint_template(SOURCE, "order.vue");
    assert_eq!(complete(&native), complete(&original));
    assert_eq!(called.load(Ordering::SeqCst), 2);
    assert_eq!(
        native.diagnostics[0].message.as_str(),
        "Component file name 'first' should be PascalCase or kebab-case"
    );
    assert_eq!(
        native.diagnostics[1].message.as_str(),
        "Component file name 'second' should be PascalCase or kebab-case"
    );
}
