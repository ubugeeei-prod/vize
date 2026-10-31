//! Original frozen history packets and genuine new partial-native controls.
mod select;
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
fn original_select_preserves_whole_native_packet_while_textarea_remains_unprovided() {
    for case in cases()
        .iter()
        .filter(|case| !case.source.starts_with("<input"))
    {
        assert_original(case, false);
        if case.source.starts_with("<select") {
            assert_original(case, true);
            continue;
        }
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

#[test]
fn duplicate_static_hidden_and_type_keep_whole_legacy_packets_but_refuse_native_headers() {
    let linter = configured();
    let original = cases()
        .into_iter()
        .find(|case| case.id == "form-label-visible-input-control")
        .unwrap();
    assert_original(&original, false);
    let warning = linter.lint_template(&original.source, "Duplicate.vue");
    for (source, name, findings) in [
        ("<input hidden hidden='until-found'>", "hidden", 0),
        ("<input hidden='until-found' hidden>", "hidden", 0),
        ("<input type='hidden' type='text'>", "type", 0),
        ("<input type='text' type='hidden'>", "type", 1),
        ("<input type type='hidden'>", "type", 1),
        ("<input type='hidden' type>", "type", 0),
    ] {
        let start = u32::try_from(source.match_indices(name).nth(1).unwrap().0).unwrap();
        let mut expected = warning.clone();
        if findings == 0 {
            expected.diagnostics.clear();
            expected.warning_count = 0;
        } else {
            expected.diagnostics.first_mut().unwrap().end = u32::try_from(source.len()).unwrap();
        }
        // The legacy parser retains both attributes and its full recoverable
        // warning. merge_lint_results orders by (start,end), so the element's
        // start-0 rule finding precedes the second attribute's parser warning.
        expected.diagnostics.push(crate::LintDiagnostic::warn(
            "parser/template",
            format!("Duplicate attribute `{name}`. Keeping the repeated attribute so parsing can continue."),
            start, start + u32::try_from(name.len()).unwrap(),
        ));
        expected.warning_count += 1;
        for _ in 0..2 {
            let legacy = linter.lint_template(source, "Duplicate.vue");
            assert_eq!(format!("{legacy:#?}"), format!("{expected:#?}"), "{source}");
            println!("native-form-label-duplicate source={source:?} legacy={legacy:#?}");
            assert_eq!(
                linter
                    .lint_native_template(source, "Duplicate.vue")
                    .unwrap_err(),
                Refusal::Header(crate::native::NativeLintRefusal::DuplicateAttribute {
                    span: Span::new(start, start + u32::try_from(name.len()).unwrap())
                })
            );
        }
    }
}

#[test]
fn closed_empty_selects_preserve_whole_findings_at_their_opening_not_body() {
    let linter = configured();
    let original = cases()
        .into_iter()
        .find(|case| case.id == "form-label-visible-input-control")
        .unwrap();
    assert_original(&original, false);
    let warning = linter.lint_template(&original.source, "Select.vue");
    let original_diagnostic = warning.diagnostics.first().unwrap();
    for prefix in ["", "<input>"] {
        for (markup, hidden) in [
            ("<select></select>", false),
            ("<select hidden></select>", true),
        ] {
            let source = format!("{prefix}{markup}");
            let mut expected = warning.clone();
            expected.diagnostics.clear();
            if !prefix.is_empty() {
                let mut input = original_diagnostic.clone();
                input.end = u32::try_from(prefix.len()).unwrap();
                expected.diagnostics.push(input);
            }
            if !hidden {
                let mut select = original_diagnostic.clone();
                select.message = "<select> elements must have an associated label".into();
                select.start = u32::try_from(prefix.len()).unwrap();
                // The ordinary markup facade reports the opening-tag range,
                // not the complete nonvoid body (MarkupElement::range).
                select.end = select.start + u32::try_from(markup.find('>').unwrap() + 1).unwrap();
                expected.diagnostics.push(select);
            }
            expected.warning_count = expected.diagnostics.len();
            for _ in 0..2 {
                let legacy = linter.lint_template(&source, "Select.vue");
                assert_eq!(format!("{legacy:#?}"), format!("{expected:#?}"), "{source}");
                println!("native-form-label-select source={source:?} legacy={legacy:#?}");
                assert_eq!(
                    format!(
                        "{:#?}",
                        linter.lint_native_template(&source, "Select.vue").unwrap()
                    ),
                    format!("{expected:#?}"),
                    "complete native {source}"
                );
            }
        }
    }
}
