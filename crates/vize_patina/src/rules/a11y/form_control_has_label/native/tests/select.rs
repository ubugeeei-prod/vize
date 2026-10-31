use super::*;
use crate::{Linter, RuleRegistry};

#[test]
fn closed_select_options_preserve_all_legacy_fields_and_unchanged_requeries() {
    for (source, findings) in [
        ("<select><option>A</option></select>", 1),
        ("<select hidden><option>A</option></select>", 0),
        (
            "<select hidden='until-found'><option>A</option></select>",
            1,
        ),
        (
            "<select hidden :hidden='false'><option>A</option></select>",
            1,
        ),
        ("<select type='hidden'><option>A</option></select>", 1),
        (
            "<select :id='name'><option :value='opaque'>A</option></select>",
            0,
        ),
        (
            "<select aria-label='Name'><option>A &amp; B</option></select>",
            0,
        ),
        ("<select title='Name'><option>A</option></select>", 0),
        ("<select :title='name'><option>A</option></select>", 1),
        (
            "<select placeholder='Name'><option>A</option><option>B</option></select>",
            1,
        ),
        (
            "<select id='  ' aria-label=''><option>A</option></select>",
            1,
        ),
        (
            "界🌸\r\n<div><input><select>\r\n<option value='A'>雪</option>\r\n</select></div>",
            2,
        ),
        ("<select></select><select hidden></select><input>", 2),
    ] {
        pair(&configured(), source, findings);
    }
}

#[test]
fn select_openings_keep_locale_help_severity_and_exact_unicode_crlf_offsets() {
    let source = "界🌸\r\n<select\r\n placeholder='Name'><option>雪</option></select>";
    let start = u32::try_from(source.find("<select").unwrap()).unwrap();
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
fn late_unclosed_implicit_unknown_and_dynamic_owners_never_return_pending_input_findings() {
    for suffix in [
        "<select>",
        "<select/>",
        "<select><option>A</option>",
        "<select><option>A</select>",
        "<select><option>A<option>B</option></select>",
        "<select><option/></select>",
        "<option>A</option>",
        "<div><option>A</option></div>",
        "<select><optgroup><option>A</option></optgroup></select>",
        "<select><select></select></select>",
        "<select><option><span>A</span></option></select>",
        "<select><div>A</div></select>",
        "<select>A</select>",
        "<select><!--A--><option>A</option></select>",
        "<select><option>{{ A }}</option></select>",
        "<select><option>{{ A</option></select>",
        "<select is='component'><option>A</option></select>",
        "<select :is='owner'><option>A</option></select>",
        "<select><option :is='owner'>A</option></select>",
        "<select :[owner]='value'><option>A</option></select>",
        "<select v-bind='attrs'><option>A</option></select>",
        "<select :type='value'><option>A</option></select>",
        "<select aria-label='&#32;'><option>A</option></select>",
        "<select><option value='A' value='B'>A</option></select>",
        "<label><select><option>A</option></select></label>",
        "<select><Widget/></select>",
        "<select><option>A</option></SELECT extra>",
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

#[test]
fn strict_callback_intersection_empty_registry_and_selected_sfc_hosts_do_not_gain_options() {
    let source = "<select hidden><option>A</option></select>";
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(super::super::super::FormControlHasLabel));
    registry.register(Box::new(crate::rules::a11y::NoAutofocus));
    let mixed = Linter::with_registry(registry);
    assert!(mixed.lint_native_template(source, "Mixed.vue").is_err());
    let disabled = mixed.with_disabled_rules(vec!["a11y/no-autofocus".into()]);
    assert!(disabled.lint_native_template(source, "One.vue").is_ok());
    assert!(
        Linter::with_registry(RuleRegistry::new())
            .lint_native_template(source, "None.vue")
            .is_err()
    );
    assert!(
        configured()
            .lint_native_sfc(&format!("<template>{source}</template>"), "Sfc.vue")
            .is_err()
    );
}
