//! Actual native attribute callbacks keep complete configured output packets.

use super::linter;
use crate::native::template::NativeTemplateLintRefusal as Refusal;
use crate::rule::{Rule, RuleRegistry};
use crate::rules::html::DeprecatedAttr;
use crate::rules::opinionated::vue::NoInlineStyle;
use crate::rules::vapor::NoInlineTemplate;
use crate::{HelpLevel, Linter, Locale, Severity};

fn family() -> Vec<Box<dyn Rule>> {
    vec![Box::new(NoInlineStyle), Box::new(DeprecatedAttr)]
}

const SOURCES: &[&str] = &[
    r#"<div style="color: red" align="center"><span bgcolor="blue"/></div>"#,
    r#"<div STYLE="red" ALIGN="center" custom:style="red" custom:align="center"/>"#,
    r#"<div style :style="value" v-bind:align.prop="value" :[field]="value"/>"#,
    r#"<div border="x"><img border="1" hspace="3"/><br clear="all"/><hr color="red"/></div>"#,
    "<div title='日本語😀'\r\n style = 'a&amp;b' align = 'center'/>\r\n<span style=\"\"/>",
    r#"<div class="x"><span>日本語😀</span></div>"#,
];

#[test]
fn actual_callbacks_preserve_complete_packets_for_all_locales_help_and_severity_options() {
    for rule in family() {
        let name = rule.meta().name;
        for locale in [Locale::En, Locale::Ja, Locale::Zh] {
            for help in [HelpLevel::Full, HelpLevel::Short, HelpLevel::None] {
                for severity in [Severity::Warning, Severity::Error] {
                    let configured = linter(
                        family()
                            .into_iter()
                            .find(|r| r.meta().name == name)
                            .unwrap(),
                    )
                    .with_locale(locale)
                    .with_help_level(help)
                    .with_rule_severity_overrides(vec![(name.into(), severity)]);
                    for source in SOURCES {
                        let expected = configured.lint_template(source, "/元/Exact.vue");
                        let actual = configured
                            .lint_native_template(source, "/元/Exact.vue")
                            .unwrap();
                        assert_eq!(
                            format!("{actual:?}"),
                            format!("{expected:?}"),
                            "{name}: {source}"
                        );
                        // Arena-independent fresh queries retain the full packet.
                        let fresh = configured
                            .lint_native_template(source, "/元/Exact.vue")
                            .unwrap();
                        assert_eq!(format!("{fresh:?}"), format!("{expected:?}"));
                    }
                }
            }
        }
    }
}

#[test]
fn actual_callback_intersection_keeps_bindings_and_strict_static_limits() {
    let mut registry = RuleRegistry::new();
    for rule in family() {
        registry.register(rule);
    }
    let both = Linter::with_registry(registry);
    for source in SOURCES {
        assert_eq!(
            format!(
                "{:?}",
                both.lint_native_template(source, "Exact.vue").unwrap()
            ),
            format!("{:?}", both.lint_template(source, "Exact.vue")),
            "{source}"
        );
    }
    let mut registry = RuleRegistry::new();
    for rule in family() {
        registry.register(rule);
    }
    registry.register(Box::new(NoInlineTemplate));
    let strict = Linter::with_registry(registry);
    assert!(matches!(
        strict.lint_native_template(r#"<div style="red" :style="value"/>"#, "Exact.vue"),
        Err(Refusal::UnsupportedAttribute { .. })
    ));
}

#[test]
fn late_comments_events_and_excluded_owners_keep_whole_input_refusal() {
    for rule in family() {
        let configured = linter(rule);
        assert!(matches!(
            configured.lint_native_template(
                r#"<div style="red" align="center"/><!-- later -->"#,
                "Exact.vue"
            ),
            Err(Refusal::Comment { .. })
        ));
        assert!(matches!(
            configured.lint_native_template(
                r#"<div style="red" align="center" @click="handler"/>"#,
                "Exact.vue"
            ),
            Err(Refusal::UnsupportedAttribute { .. })
        ));
        assert!(matches!(
            configured
                .lint_native_template(r#"<div style="red" align="center"><p/></div>"#, "Exact.vue"),
            Err(Refusal::UnsupportedContext { .. })
        ));
        assert!(matches!(
            configured.lint_native_template(
                r#"<div style="red" align="center">{{ value }}</div>"#,
                "Exact.vue"
            ),
            Err(Refusal::Interpolation { .. })
        ));
    }
}
