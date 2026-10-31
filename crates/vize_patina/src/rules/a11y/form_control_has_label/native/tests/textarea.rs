use super::*;

#[test]
fn closed_plain_textarea_headers_match_every_legacy_field_and_unchanged_requery() {
    for (source, findings) in [
        ("<textarea></textarea>", 1),
        ("<textarea hidden></textarea>", 0),
        ("<textarea hidden='HIDDEN'>plain</textarea>", 0),
        ("<textarea hidden='until-found'>plain</textarea>", 1),
        ("<textarea hidden :hidden='false'>plain</textarea>", 1),
        ("<textarea type='hidden'>plain</textarea>", 1),
        ("<textarea placeholder='Name'>plain</textarea>", 1),
        ("<textarea id='name'>雪🌸\r\nplain</textarea>", 0),
        ("<textarea aria-label='Name'>plain</textarea>", 0),
        ("<textarea aria-labelledby='name'>plain</textarea>", 0),
        ("<textarea title='Name'>plain</textarea>", 0),
        ("<textarea :id='name'>plain</textarea>", 0),
        ("<textarea :aria-label='name'>plain</textarea>", 0),
        ("<textarea :aria-labelledby='name'>plain</textarea>", 0),
        ("<textarea :title='name'>plain</textarea>", 1),
        ("<textarea aria-label='  ' title=''>\nplain</textarea>", 1),
        (
            "<input><select><option>A</option></select><textarea>plain</textarea>",
            3,
        ),
    ] {
        pair(&configured(), source, findings);
    }
}

#[test]
fn plain_textarea_diagnostics_use_only_original_openings_in_all_locales_and_policies() {
    let source = "界🌸\r\n<textarea\r\n placeholder='Name'>\r\n雪\r\n</textarea>";
    let start = u32::try_from(source.find("<textarea").unwrap()).unwrap();
    let end = u32::try_from(source.find('>').unwrap() + 1).unwrap();
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for severity in [Severity::Warning, Severity::Error] {
                let linter = configured()
                    .with_locale(locale)
                    .with_help_level(help)
                    .with_rule_severity_overrides(vec![(RULE.into(), severity)]);
                pair(&linter, source, 1);
                let result = linter.lint_native_template(source, "NewPair.vue").unwrap();
                let diagnostic = &result.diagnostics[0];
                assert_eq!((diagnostic.start, diagnostic.end), (start, end));
                assert!(diagnostic.labels.is_empty());
                assert!(diagnostic.fix.is_none());
            }
        }
    }
}

#[test]
fn raw_markup_entities_markers_unknown_owners_and_recovery_refuse_all_pending_findings() {
    for suffix in [
        "<textarea>",
        "<textarea/>",
        "<textarea>plain",
        "<textarea>plain</div>",
        "<textarea><input></textarea>",
        "<textarea><span>plain</span></textarea>",
        "<textarea>&amp;</textarea>",
        "<textarea>&#32;</textarea>",
        "<textarea>{{ value }}</textarea>",
        "<textarea>{{ value</textarea>",
        "<textarea><!--opaque--></textarea>",
        "<textarea hidden><input></textarea>",
        "<textarea hidden>&amp;</textarea>",
        "<textarea hidden>{{value}}</textarea>",
        "<textarea is='component'></textarea>",
        "<textarea :is='owner'></textarea>",
        "<textarea :[owner]='value'></textarea>",
        "<textarea v-bind='attrs'></textarea>",
        "<textarea :type='value'></textarea>",
        "<textarea aria-label='&#32;'></textarea>",
        "<textarea hidden hidden></textarea>",
        "<label><textarea hidden></textarea></label>",
        "<select><textarea hidden></textarea></select>",
        "<select><option><textarea hidden></textarea></option></select>",
    ] {
        let source = format!("<input>{suffix}");
        for _ in 0..2 {
            assert!(
                configured()
                    .lint_native_template(&source, "Refuse.vue")
                    .is_err(),
                "{source}"
            );
        }
    }
}
