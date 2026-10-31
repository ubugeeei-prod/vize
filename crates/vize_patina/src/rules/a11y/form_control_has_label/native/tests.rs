//! Original frozen history packets and genuine new partial-native controls.
mod support;

use crate::native::template::NativeTemplateLintRefusal as Refusal;
use crate::{HelpLevel, Locale, Severity};
use support::*;
use vize_l0::{Span, config::VueVersion};

#[test]
fn four_original_input_packets_preserve_all_fields_fix_absence_and_unchanged_requery() {
    let cases = cases();
    let inputs: Vec<_> = cases
        .iter()
        .filter(|case| case.source.starts_with("<input"))
        .collect();
    assert_eq!(inputs.len(), 4);
    for case in inputs {
        assert_original(case, false);
        assert_original(case, true);
    }
}

#[test]
fn original_select_and_textarea_keep_complete_legacy_packets_and_context_refusals() {
    for case in cases()
        .iter()
        .filter(|case| !case.source.starts_with("<input"))
    {
        assert_original(case, false);
        let end = u32::try_from(case.source.find('>').unwrap() + 1).unwrap();
        for _ in 0..2 {
            assert_eq!(
                configured()
                    .lint_native_template(&case.source, &case.filename)
                    .unwrap_err(),
                Refusal::UnsupportedContext {
                    span: Span::new(0, end)
                }
            );
        }
    }
}

#[test]
fn static_exemptions_labels_and_bound_label_names_match_the_complete_legacy_contract() {
    for (source, findings) in [
        ("<input>", 1),
        (
            "日本語🌸\r\n<div><input type='text'/><input placeholder='Email'></div>",
            2,
        ),
        ("<input hidden='' type='file'>", 0),
        ("<input hidden='HIDDEN'>", 0),
        ("<input hidden='until-found'>", 1),
        ("<input hidden :hidden='false'>", 1),
        (
            "<input type='hidden'><input type='submit'><input type='reset'><input type='button'><input type='image'>",
            0,
        ),
        ("<input type='HIDDEN'>", 1),
        (
            "<input id='name'><input title='Name'><input aria-label='Name'><input aria-labelledby='label'>",
            0,
        ),
        (
            "<input :id='name'><input :aria-label='name'><input :aria-labelledby='name'>",
            0,
        ),
        (
            "<input id='  ' title='' aria-label='\t' aria-labelledby=''>",
            1,
        ),
        ("<input :title='name' :placeholder='name'>", 1),
        (
            "<div title='parent'><span><input type='text'></span></div>",
            1,
        ),
        (
            "<input .id='name'><input v-bind:aria-label.camel='name'>",
            0,
        ),
    ] {
        pair(&configured(), source, findings);
    }
}

#[test]
fn all_locale_help_and_severity_controls_compare_whole_original_outputs_and_requery() {
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for severity in [Severity::Warning, Severity::Error] {
                let linter = configured()
                    .with_locale(locale)
                    .with_help_level(help)
                    .with_rule_severity_overrides(vec![(RULE.into(), severity)]);
                pair(&linter, "<input placeholder='Email'><input type='text'>", 2);
            }
        }
    }
}

#[test]
fn late_unknown_label_type_dynamic_name_and_encoded_values_refuse_the_entire_result() {
    for (suffix, attribute) in [
        ("<label><input></label>", None),
        ("<input :type=\"'hidden'\">", Some(":type=\"'hidden'\"")),
        ("<input :[id]='name'>", Some(":[id]='name'")),
        ("<input aria-label='&#32;'>", Some("aria-label='&#32;'")),
        ("<input title='a&amp;b'>", Some("title='a&amp;b'")),
    ] {
        let source = format!("<input type='text'>{suffix}");
        let expected = if let Some(attribute) = attribute {
            let start = u32::try_from(source.find(attribute).unwrap()).unwrap();
            Refusal::UnsupportedAttribute {
                span: Span::new(start, start + u32::try_from(attribute.len()).unwrap()),
            }
        } else {
            let start = u32::try_from(source.find("<label>").unwrap()).unwrap();
            Refusal::UnsupportedContext {
                span: Span::new(start, start + 7),
            }
        };
        for _ in 0..2 {
            assert_eq!(
                configured()
                    .lint_native_template(&source, "Late.vue")
                    .unwrap_err(),
                expected
            );
        }
    }
}

#[test]
fn late_whole_grammar_refusals_and_rule_selection_remain_fail_closed() {
    for suffix in [
        "<!-- later -->",
        "{{ later }}",
        "<Widget/>",
        "<input v-bind='attrs'>",
    ] {
        let source = format!("<input>{suffix}");
        assert!(
            configured()
                .lint_native_template(&source, "Later.vue")
                .is_err()
        );
    }
    pair(
        &configured().with_disabled_rules(vec![RULE.into()]),
        "<input>",
        0,
    );
    assert_eq!(
        configured()
            .with_vue_version(Some(VueVersion::V2))
            .lint_native_template("<input>", "Legacy.vue")
            .unwrap_err(),
        Refusal::UnsupportedVueVersion {
            requested: VueVersion::V2
        }
    );
    assert_eq!(
        configured()
            .with_vapor_mode(Some(true))
            .lint_native_template("<input>", "Vapor.vue")
            .unwrap_err(),
        Refusal::UnsupportedVaporMode { requested: true }
    );
    assert_eq!(
        configured()
            .with_enabled_rules(Some(vec![RULE.into(), "vue/valid-v-if".into()]))
            .lint_native_template("<input>", "Unprovided.vue")
            .unwrap_err(),
        Refusal::UnprovidedRule {
            rule: "vue/valid-v-if".into()
        }
    );
}
