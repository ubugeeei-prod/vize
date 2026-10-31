//! Original template-only SFC custody and complete public slot observations.
//! The frozen history oracle remains legacy evidence until source execution.

mod configuration;
mod refusal;
mod support;

use support::*;
use vize_patina::{HelpLevel, Locale, Severity};

#[test]
fn frozen_history_has_the_complete_original_result_and_unchanged_requery() {
    let case = history_case("slot-compat-vue3");
    assert_eq!(
        serde_json::to_value(&case).unwrap(),
        serde_json::json!({
            "id": "slot-compat-vue3", "history": "f9fa82f7867e3a9373a8d0ee30162c6947c7c113",
            "source": SOURCE, "filename": "History.vue", "entry": "sfc",
            "rule": RULE, "vue_version": "3", "vapor": null, "diagnostics": 1, "fixes": 0,
        })
    );
    let linter = configured(Locale::En, HelpLevel::Full);
    let expected = expected(
        "History.vue",
        15,
        Severity::Error,
        Locale::En,
        HelpLevel::Full,
    );
    assert_pair(&linter, SOURCE, "History.vue", &expected);

    let frozen = include_str!("snapshots/integration__lint_history__slot-compat-vue3.snap")
        .split_once("\n---\n")
        .unwrap()
        .1;
    for native in [false, true] {
        let observation = observe(&linter, &case, native);
        assert_eq!(complete(&observation.initial), expected);
        assert!(observation.applications.is_empty());
        assert_eq!(
            complete(observation.unchanged_requery.as_ref().unwrap()),
            expected
        );
        assert_eq!(format!("{case:#?}\n{observation:#?}\n"), frozen);
    }
}

#[test]
fn original_source_offsets_survive_whitespace_prefix_and_utf8_content() {
    for source in [
        "\r\n\t<template>\r\n<div slot=\"header\">Title</div>\r\n</template>\r\n",
        "\n<template>日本語🍣<div slot=\"header\">Title</div></template>\n",
    ] {
        let start = u32::try_from(source.find("slot=").unwrap()).unwrap();
        assert!(start > 15);
        let linter = configured(Locale::En, HelpLevel::Full);
        assert_pair(
            &linter,
            source,
            "nested/Prefix.vue",
            &expected(
                "nested/Prefix.vue",
                start,
                Severity::Error,
                Locale::En,
                HelpLevel::Full,
            ),
        );
    }
}

#[test]
fn configured_severity_locale_and_help_preserve_every_result_field() {
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for severity in [Severity::Error, Severity::Warning] {
                let linter = configured(locale, help)
                    .with_rule_severity_overrides(vec![(RULE.into(), severity)]);
                assert_pair(
                    &linter,
                    SOURCE,
                    "configured.vue",
                    &expected("configured.vue", 15, severity, locale, help),
                );
            }
        }
    }
}
