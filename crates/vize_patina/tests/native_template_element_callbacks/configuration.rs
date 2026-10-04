use super::support::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use vize_l0::config::VueVersion;
use vize_l1::markup::NativeLintComponent;
use vize_patina::{
    HelpLevel, Linter, Locale, Rule, RuleMeta, RuleRegistry, Severity,
    native::{
        NativeLintRefusal,
        template::{
            NativeTemplateAttributeProfile as Profile, NativeTemplateLintContext,
            NativeTemplateLintRefusal as Refusal, NativeTemplateRule,
        },
    },
};

struct RootOnly;
impl Rule for RootOnly {
    fn meta(&self) -> &'static RuleMeta {
        &FIRST
    }
    fn as_native_template_rule(&self) -> Option<&dyn NativeTemplateRule> {
        Some(self)
    }
}
impl NativeTemplateRule for RootOnly {
    fn run_on_template<'a>(
        &self,
        _: &mut NativeTemplateLintContext<'_, 'a>,
        _: &NativeLintComponent<'a>,
    ) -> Result<(), Refusal> {
        Ok(())
    }
}
struct Namesake;
impl Rule for Namesake {
    fn meta(&self) -> &'static RuleMeta {
        &SECOND
    }
}

#[test]
fn empty_and_default_root_callbacks_preserve_static_only_admission_and_noop_output() {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(RootOnly));
    for linter in [
        Linter::with_registry(registry),
        Linter::with_registry(RuleRegistry::new()),
    ] {
        let result = pair(
            &linter,
            "<div id='literal'><span title='text'/></div>",
            "exact.vue",
        );
        assert_eq!(
            complete(&result),
            serde_json::json!({
                "filename": "exact.vue", "diagnostics": [], "error_count": 0, "warning_count": 0,
            })
        );
        let source = "<div :title='opaque'></div>";
        assert_eq!(
            linter
                .lint_native_template(source, "exact.vue")
                .unwrap_err(),
            Refusal::UnsupportedAttribute {
                span: span(source, ":title='opaque'")
            }
        );
        let source = "<div :[name]='opaque'></div>";
        assert_eq!(
            linter
                .lint_native_template(source, "exact.vue")
                .unwrap_err(),
            Refusal::Header(NativeLintRefusal::UnresolvedBinding {
                span: span(source, ":[name]")
            })
        );
    }
}

#[test]
fn every_actual_enabled_instance_must_admit_bindings_in_either_registry_order() {
    for reverse in [false, true] {
        let log = events();
        let wide = Audit::new(&FIRST, "wide", Profile::Bindings, log.clone());
        let narrow = Audit::new(&SECOND, "narrow", Profile::StaticOnly, log.clone());
        let wide_calls = wide.profile_calls.clone();
        let narrow_calls = narrow.profile_calls.clone();
        let rules = if reverse {
            vec![narrow, wide]
        } else {
            vec![wide, narrow]
        };
        let configured = linter(rules, Locale::En, HelpLevel::Full);
        let source = "<div .title='opaque'></div>";
        assert_eq!(
            configured
                .lint_native_template(source, "exact.vue")
                .unwrap_err(),
            Refusal::UnsupportedAttribute {
                span: span(source, ".title='opaque'")
            }
        );
        assert_eq!(wide_calls.load(Ordering::SeqCst), 1);
        assert_eq!(narrow_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            *log.lock().unwrap(),
            if reverse {
                vec![Event::Root("narrow"), Event::Root("wide")]
            } else {
                vec![Event::Root("wide"), Event::Root("narrow")]
            }
        );
    }
}

#[test]
fn disabled_instance_does_not_narrow_or_receive_callbacks_but_enabled_order_is_exact() {
    let log = events();
    let wide = Audit::new(&FIRST, "wide", Profile::Bindings, log.clone());
    let narrow = Audit::new(&SECOND, "disabled", Profile::StaticOnly, log.clone());
    let wide_calls = wide.profile_calls.clone();
    let narrow_calls = narrow.profile_calls.clone();
    let configured = linter(vec![narrow, wide], Locale::En, HelpLevel::Full)
        .with_enabled_rules(Some(vec![FIRST.name.into(), SECOND.name.into()]))
        .with_disabled_rules(vec![SECOND.name.into()]);
    let source = "<div :[name]='opaque'></div>";
    pair(&configured, source, "exact.vue");
    assert_eq!(
        wide_calls.load(Ordering::SeqCst),
        2,
        "one profile call per native invocation"
    );
    assert_eq!(narrow_calls.load(Ordering::SeqCst), 0);
    let once = vec![
        Event::Root("wide"),
        element_event("wide", "div", 0, None),
        binding_event(
            "wide",
            source,
            0,
            ":[name]",
            ":[name]='opaque'",
            "opaque",
            vize_l1::markup::ArgSyntax::Dynamic(span(source, "name")),
        ),
    ];
    assert_eq!(*log.lock().unwrap(), [once.clone(), once].concat());
}

#[test]
fn root_side_effect_cannot_widen_the_already_frozen_actual_profile() {
    let log = events();
    let widened = Arc::new(AtomicBool::new(false));
    let mut rule = Audit::new(&FIRST, "changing", Profile::StaticOnly, log.clone());
    rule.widen = Some(widened.clone());
    let calls = rule.profile_calls.clone();
    let configured = linter(vec![rule], Locale::En, HelpLevel::Full);
    let source = "<div :title='opaque'></div>";
    assert_eq!(
        configured
            .lint_native_template(source, "exact.vue")
            .unwrap_err(),
        Refusal::UnsupportedAttribute {
            span: span(source, ":title='opaque'")
        }
    );
    assert!(widened.load(Ordering::SeqCst));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(*log.lock().unwrap(), [Event::Root("changing")]);
}

#[test]
fn unprovided_actual_namesake_or_unknown_enabled_rule_refuses_before_profiles_and_output() {
    let log = events();
    let rule = Audit::new(&FIRST, "actual", Profile::Bindings, log.clone());
    let calls = rule.profile_calls.clone();
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(rule));
    registry.register(Box::new(Namesake));
    let configured = Linter::with_registry(registry);
    assert_eq!(
        configured
            .lint_native_template("<div/>", "exact.vue")
            .unwrap_err(),
        Refusal::UnprovidedRule {
            rule: SECOND.name.into()
        }
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(log.lock().unwrap().is_empty());
    let log = events();
    let configured = linter(
        vec![Audit::new(&FIRST, "actual", Profile::Bindings, log.clone())],
        Locale::En,
        HelpLevel::Full,
    )
    .with_enabled_rules(Some(vec![FIRST.name.into(), "unknown/native".into()]));
    assert_eq!(
        configured
            .lint_native_template("<div/>", "exact.vue")
            .unwrap_err(),
        Refusal::UnprovidedRule {
            rule: "unknown/native".into()
        }
    );
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn configured_locale_help_severity_and_unsorted_root_element_attribute_order_match_full_original() {
    let source = "界\r\n<div id='&amp;' :title.camel='opaque'><span .value='raw'/></div>";
    for locale in LOCALES {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            let log = events();
            let configured = linter(
                vec![
                    Audit::new(&SECOND, "second", Profile::Bindings, log.clone()),
                    Audit::new(&FIRST, "first", Profile::Bindings, log),
                ],
                locale,
                help,
            )
            .with_rule_severity_overrides(vec![(FIRST.name.into(), Severity::Error)]);
            let result = pair(&configured, source, "/元/Exact.File.vue");
            assert_eq!(result.error_count, 4);
            assert_eq!(result.warning_count, 4);
            let ranges = [
                vize_l0::Span::new(0, 0),
                vize_l0::Span::new(0, 0),
                span(source, "id='&amp;'"),
                span(source, ":title.camel='opaque'"),
                span(source, "id='&amp;'"),
                span(source, ":title.camel='opaque'"),
                span(source, ".value='raw'"),
                span(source, ".value='raw'"),
            ];
            let names = [
                SECOND.name,
                FIRST.name,
                SECOND.name,
                SECOND.name,
                FIRST.name,
                FIRST.name,
                SECOND.name,
                FIRST.name,
            ];
            for ((diagnostic, range), name) in result.diagnostics.iter().zip(ranges).zip(names) {
                assert_eq!(
                    (diagnostic.rule_name, diagnostic.start, diagnostic.end),
                    (name, range.start, range.end)
                );
                assert_eq!(
                    diagnostic.severity,
                    if name == FIRST.name {
                        Severity::Error
                    } else {
                        Severity::Warning
                    }
                );
                assert_eq!(diagnostic.help.is_none(), help == HelpLevel::None);
                assert!(diagnostic.labels.is_empty());
                assert!(diagnostic.fix.is_none());
            }
        }
    }
}

#[test]
fn unsupported_host_dialect_options_refuse_before_any_actual_profile_or_callback() {
    for (version, vapor, refusal) in [
        (
            Some(VueVersion::V2),
            None,
            Refusal::UnsupportedVueVersion {
                requested: VueVersion::V2,
            },
        ),
        (
            Some(VueVersion::V2_7),
            None,
            Refusal::UnsupportedVueVersion {
                requested: VueVersion::V2_7,
            },
        ),
        (
            None,
            Some(true),
            Refusal::UnsupportedVaporMode { requested: true },
        ),
    ] {
        let log = events();
        let rule = Audit::new(&FIRST, "actual", Profile::Bindings, log.clone());
        let calls = rule.profile_calls.clone();
        let configured = linter(vec![rule], Locale::En, HelpLevel::Full)
            .with_vue_version(version)
            .with_vapor_mode(vapor);
        assert_eq!(
            configured
                .lint_native_template("<div/>", "exact.vue")
                .unwrap_err(),
            refusal
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(log.lock().unwrap().is_empty());
    }
}

mod unprovided;
