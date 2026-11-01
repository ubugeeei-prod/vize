mod support;
use crate::native::template::NativeTemplateLintRefusal as Refusal;
use crate::{HelpLevel, Locale, Severity};
use support::*;
use vize_l0::Span;

#[test]
fn all_four_originals_keep_every_legacy_byte_three_direct_values_match_native_and_optgroup_refuses()
{
    for case in cases() {
        assert_original(&case, false);
        if case.id == "placeholder-label-optgroup-report" {
            for _ in 0..2 {
                assert_eq!(
                    configured()
                        .lint_native_template(&case.source, &case.filename)
                        .unwrap_err(),
                    Refusal::UnsupportedContext {
                        span: Span::new(17, 37)
                    }
                );
            }
        } else {
            assert_original(&case, true);
        }
    }
}

#[test]
fn static_required_single_size_and_first_explicit_value_keep_complete_legacy_results() {
    for (source, findings) in [
        ("<select><option value=''>Any</option></select>", 0),
        (
            "<select required><option value=''>Choose</option><option value='a'>A</option></select>",
            0,
        ),
        ("<select required><option value>Choose</option></select>", 0),
        (
            "<select required><option value='a'>A</option><option value=''>Choose</option></select>",
            1,
        ),
        (
            "<select required><option value=' '>Whitespace is a value</option></select>",
            1,
        ),
        (
            "<select required><option value='雪'>Snow</option></select>",
            1,
        ),
        (
            "<select required='false'><option value='a'>A</option></select>",
            1,
        ),
        (
            "<select required multiple><option value='a'>A</option></select>",
            0,
        ),
        (
            "<select required multiple='false'><option value='a'>A</option></select>",
            0,
        ),
        (
            "<select required size=' 1 '><option value='a'>A</option></select>",
            1,
        ),
        (
            "<select required size='01'><option value='a'>A</option></select>",
            0,
        ),
        (
            "<select required size='3'><option value='a'>A</option></select>",
            0,
        ),
        (
            "<select required size><option value='a'>A</option></select>",
            0,
        ),
        ("<select required></select>", 0),
        (
            "<select required><option value='a'>A</option></select><select required><option value='b'>B</option></select>",
            2,
        ),
    ] {
        pair(&configured(), source, findings);
    }
}

#[test]
fn original_option_opening_not_select_or_body_has_exact_offsets_and_all_locale_policies() {
    let source = "界🌸\r\n<select required>\r\n<option\r\n value='a'>雪</option></select>";
    let start = u32::try_from(source.find("<option").unwrap()).unwrap();
    let end =
        u32::try_from(source[start as usize..].find('>').unwrap() + start as usize + 1).unwrap();
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for severity in [Severity::Warning, Severity::Error] {
                let linter = configured()
                    .with_locale(locale)
                    .with_help_level(help)
                    .with_rule_severity_overrides(vec![(RULE.into(), severity)]);
                pair(&linter, source, 1);
                let actual = linter.lint_native_template(source, "Pair.vue").unwrap();
                assert_eq!(
                    (actual.diagnostics[0].start, actual.diagnostics[0].end),
                    (start, end)
                );
            }
        }
    }
}

#[test]
fn text_derived_bound_entity_grouped_unknown_or_late_invalid_values_never_return_partial_output() {
    let prefix = "<select required><option value='a'>A</option></select>";
    for suffix in [
        "<select required><option>Choose</option></select>",
        "<select required><option></option></select>",
        "<select><option>Any</option></select>",
        "<select required><option :value=\"''\">Choose</option></select>",
        "<select required><option :value='first'>First</option></select>",
        "<select :required='strict'><option value='a'>A</option></select>",
        "<select required :size=\"'1'\"><option value='a'>A</option></select>",
        "<select required size='&#49;'><option value='a'>A</option></select>",
        "<select required><option value='&#32;'>Space</option></select>",
        "<select required><option value='a'>A</option><option value='&amp;'>Amp</option></select>",
        "<select required><option value='a'>A</option><option>Unknown value</option></select>",
        "<select required><optgroup label='G'><option value=''>Choose</option></optgroup></select>",
        "<select required><option value='a'>{{ value }}</option></select>",
        "<select required><option value='a' v-if='ok'>A</option></select>",
        "<select required><slot/></select>",
        "<select required><MyOption value='a'/></select>",
        "<select required><option value='a'>A</select>",
        "<select required><option value='a'/></select>",
        "<select required><option value='a' value='b'>A</option></select>",
    ] {
        let source = format!("{prefix}{suffix}");
        for _ in 0..2 {
            assert!(
                configured()
                    .lint_native_template(&source, "Late.vue")
                    .is_err(),
                "{source}"
            );
        }
    }
}

#[test]
fn actual_callback_selection_and_original_host_modes_remain_fail_closed() {
    use vize_l0::config::VueVersion;
    pair(
        &configured().with_disabled_rules(vec![RULE.into()]),
        "<input>",
        0,
    );
    let source = "<select required><option value='a'>A</option></select>";
    assert_eq!(
        configured()
            .with_enabled_rules(Some(vec![RULE.into(), "unprovided/rule".into()]))
            .lint_native_template(source, "Unknown.vue")
            .unwrap_err(),
        Refusal::UnprovidedRule {
            rule: "unprovided/rule".into()
        }
    );
    assert_eq!(
        configured()
            .with_vue_version(Some(VueVersion::V2))
            .lint_native_template(source, "Vue2.vue")
            .unwrap_err(),
        Refusal::UnsupportedVueVersion {
            requested: VueVersion::V2
        }
    );
    assert_eq!(
        configured()
            .with_vapor_mode(Some(true))
            .lint_native_template(source, "Vapor.vue")
            .unwrap_err(),
        Refusal::UnsupportedVaporMode { requested: true }
    );
    assert!(
        configured()
            .lint_native_sfc(&format!("<template>{source}</template>"), "Sfc.vue")
            .is_err()
    );
}
