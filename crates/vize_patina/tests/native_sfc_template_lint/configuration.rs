use super::support::{RULE, SOURCE};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use vize_l0::Span;
use vize_patina::{
    Linter, Rule, RuleCategory, RuleMeta, RuleRegistry, Severity,
    native::{
        sfc::{
            NativeSfcLintRefusal, NativeSfcTemplateLintContext, NativeSfcTemplateOwner,
            NativeSfcTemplateRule,
        },
        template::{
            NativeTemplateAttributeKind, NativeTemplateElement, NativeTemplateLintRefusal,
            NativeTemplateTextProfile,
        },
    },
    rules::vue::NoDeprecatedSlotAttribute,
};

static PROBE_META: RuleMeta = RuleMeta {
    name: "test/native-sfc-template-probe",
    description: "Observe authentic configured callbacks and pending findings",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
struct Calls {
    profiles: AtomicUsize,
    roots: AtomicUsize,
    elements: AtomicUsize,
    reports: AtomicUsize,
}

struct Probe {
    literal: Arc<AtomicBool>,
    narrow_after_freeze: bool,
    report: bool,
    calls: Arc<Calls>,
}

impl Rule for Probe {
    fn meta(&self) -> &'static RuleMeta {
        &PROBE_META
    }
    fn as_native_sfc_template_rule(&self) -> Option<&dyn NativeSfcTemplateRule> {
        Some(self)
    }
}

impl NativeSfcTemplateRule for Probe {
    fn text_profile(&self) -> NativeTemplateTextProfile {
        self.calls.profiles.fetch_add(1, Ordering::SeqCst);
        if self.literal.load(Ordering::SeqCst) {
            NativeTemplateTextProfile::LiteralText
        } else {
            NativeTemplateTextProfile::WhitespaceOnly
        }
    }

    fn run_on_template<'a>(
        &self,
        context: &mut NativeSfcTemplateLintContext<'_, 'a>,
        owner: &NativeSfcTemplateOwner<'a>,
    ) -> Result<(), NativeTemplateLintRefusal> {
        assert!(core::ptr::eq(context.owner(), owner));
        assert!(core::ptr::eq(context.source(), owner.source()));
        assert!(core::ptr::eq(
            context.source(),
            owner.template().component().block().root_source()
        ));
        assert_eq!(context.filename(), "Probe.vue");
        self.calls.roots.fetch_add(1, Ordering::SeqCst);
        if self.narrow_after_freeze {
            self.literal.store(false, Ordering::SeqCst);
        }
        Ok(())
    }

    fn run_on_element<'a>(
        &self,
        context: &mut NativeSfcTemplateLintContext<'_, 'a>,
        element: &NativeTemplateElement<'_, 'a>,
    ) -> Result<(), NativeTemplateLintRefusal> {
        assert!(core::ptr::eq(
            context.owner().template().component(),
            element.original().component()
        ));
        self.calls.elements.fetch_add(1, Ordering::SeqCst);
        for attribute in element.attributes() {
            if self.report
                && matches!(
                    attribute.kind(),
                    NativeTemplateAttributeKind::Static { name: "slot", .. }
                )
            {
                context.error_attribute_name_with_help(
                    attribute,
                    "vue/no-deprecated-slot-attribute.message",
                    &[],
                    "vue/no-deprecated-slot-attribute.help",
                )?;
                self.calls.reports.fetch_add(1, Ordering::SeqCst);
            }
        }
        Ok(())
    }
}

fn configured_probe(
    reverse: bool,
    literal: bool,
    narrow_after_freeze: bool,
    report: bool,
) -> (Linter, Arc<Calls>) {
    let calls = Arc::new(Calls::default());
    let probe = Box::new(Probe {
        literal: Arc::new(AtomicBool::new(literal)),
        narrow_after_freeze,
        report,
        calls: calls.clone(),
    });
    let mut registry = RuleRegistry::new();
    if reverse {
        registry.register(probe);
        registry.register(Box::new(NoDeprecatedSlotAttribute));
    } else {
        registry.register(Box::new(NoDeprecatedSlotAttribute));
        registry.register(probe);
    }
    (Linter::with_registry(registry), calls)
}

#[test]
fn literal_text_requires_every_actual_callback_in_either_registry_order() {
    for reverse in [false, true] {
        let (linter, calls) = configured_probe(reverse, false, false, false);
        let start = u32::try_from(SOURCE.find("Title").unwrap()).unwrap();
        assert_eq!(
            linter.lint_native_sfc(SOURCE, "Probe.vue").unwrap_err(),
            NativeSfcLintRefusal::Template(NativeTemplateLintRefusal::UnsupportedText {
                span: Span::new(start, start + 5),
            }),
        );
        assert_eq!(calls.profiles.load(Ordering::SeqCst), 1);
        assert_eq!(calls.roots.load(Ordering::SeqCst), 1);
        assert_eq!(calls.elements.load(Ordering::SeqCst), 1);
        let whitespace = "<template><div slot='header'> \r\n\t</div></template>";
        let native = linter.lint_native_sfc(whitespace, "Probe.vue").unwrap();
        let original = linter.lint_sfc(whitespace, "Probe.vue");
        assert_eq!(format!("{native:#?}"), format!("{original:#?}"));
    }
}

#[test]
fn the_default_capability_does_not_grant_literal_text() {
    struct DefaultText;
    impl Rule for DefaultText {
        fn meta(&self) -> &'static RuleMeta {
            &PROBE_META
        }
        fn as_native_sfc_template_rule(&self) -> Option<&dyn NativeSfcTemplateRule> {
            Some(self)
        }
    }
    impl NativeSfcTemplateRule for DefaultText {
        fn run_on_template<'a>(
            &self,
            _: &mut NativeSfcTemplateLintContext<'_, 'a>,
            _: &NativeSfcTemplateOwner<'a>,
        ) -> Result<(), NativeTemplateLintRefusal> {
            Ok(())
        }
    }
    for reverse in [false, true] {
        let mut registry = RuleRegistry::new();
        if reverse {
            registry.register(Box::new(DefaultText));
            registry.register(Box::new(NoDeprecatedSlotAttribute));
        } else {
            registry.register(Box::new(NoDeprecatedSlotAttribute));
            registry.register(Box::new(DefaultText));
        }
        assert!(matches!(
            Linter::with_registry(registry).lint_native_sfc(SOURCE, "Probe.vue"),
            Err(NativeSfcLintRefusal::Template(
                NativeTemplateLintRefusal::UnsupportedText { .. }
            )),
        ));
    }
}

#[test]
fn profile_is_frozen_once_before_callbacks_can_mutate_the_next_query() {
    let (linter, calls) = configured_probe(false, true, true, false);
    let original = linter.lint_sfc(SOURCE, "Probe.vue");
    let native = linter.lint_native_sfc(SOURCE, "Probe.vue").unwrap();
    assert_eq!(format!("{native:#?}"), format!("{original:#?}"));
    assert_eq!(calls.profiles.load(Ordering::SeqCst), 1);
    assert_eq!(calls.roots.load(Ordering::SeqCst), 1);
    assert!(matches!(
        linter.lint_native_sfc(SOURCE, "Probe.vue"),
        Err(NativeSfcLintRefusal::Template(
            NativeTemplateLintRefusal::UnsupportedText { .. }
        )),
    ));
    assert_eq!(calls.profiles.load(Ordering::SeqCst), 2);
    assert_eq!(calls.roots.load(Ordering::SeqCst), 2);
}

#[test]
fn whole_query_refusal_discards_findings_reported_before_a_later_child() {
    for (suffix, refused, kind) in [
        (
            "<!-- later comment -->",
            "<!-- later comment -->",
            "comment",
        ),
        (
            "<!-- eslint-disable vue/no-deprecated-slot-attribute -->",
            "<!-- eslint-disable vue/no-deprecated-slot-attribute -->",
            "comment",
        ),
        ("{{ later }}", "{{ later }}", "interpolation"),
        ("<table></table>", "<table>", "context"),
        ("<svg></svg>", "<svg>", "context"),
    ] {
        let source = format!("<template><div slot='header'>Title</div>{suffix}</template>");
        let (linter, calls) = configured_probe(false, true, false, true);
        let refusal = linter.lint_native_sfc(&source, "Probe.vue").unwrap_err();
        let start = u32::try_from(source.find(refused).unwrap()).unwrap();
        let span = Span::new(start, start + u32::try_from(refused.len()).unwrap());
        let expected = match kind {
            "comment" => NativeTemplateLintRefusal::Comment { span },
            "interpolation" => NativeTemplateLintRefusal::Interpolation { span },
            _ => NativeTemplateLintRefusal::UnsupportedContext { span },
        };
        assert_eq!(
            refusal,
            NativeSfcLintRefusal::Template(expected),
            "{suffix}"
        );
        // This increment occurs only after the checked reporter accepted the
        // first genuine attribute. Early parser refusal cannot satisfy the law.
        assert_eq!(calls.reports.load(Ordering::SeqCst), 1, "{suffix}");
        assert_eq!(calls.elements.load(Ordering::SeqCst), 1, "{suffix}");
        assert_eq!(calls.roots.load(Ordering::SeqCst), 1, "{suffix}");
        assert_eq!(calls.profiles.load(Ordering::SeqCst), 1, "{suffix}");
    }
}

#[test]
fn a_disabled_narrow_callback_contributes_neither_profile_nor_invocation() {
    let (linter, calls) = configured_probe(false, false, false, false);
    let linter = linter.with_disabled_rules(vec![PROBE_META.name.into()]);
    let native = linter.lint_native_sfc(SOURCE, "Probe.vue").unwrap();
    let original = linter.lint_sfc(SOURCE, "Probe.vue");
    assert_eq!(format!("{native:#?}"), format!("{original:#?}"));
    assert_eq!(calls.profiles.load(Ordering::SeqCst), 0);
    assert_eq!(calls.roots.load(Ordering::SeqCst), 0);
}

#[test]
fn a_catalog_name_without_the_configured_capability_refuses() {
    struct Namesake;
    impl Rule for Namesake {
        fn meta(&self) -> &'static RuleMeta {
            <NoDeprecatedSlotAttribute as Rule>::meta(&NoDeprecatedSlotAttribute)
        }
    }
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(Namesake));
    assert_eq!(
        Linter::with_registry(registry)
            .lint_native_sfc(SOURCE, "Probe.vue")
            .unwrap_err(),
        NativeSfcLintRefusal::UnprovidedRule { rule: RULE.into() },
    );
}
