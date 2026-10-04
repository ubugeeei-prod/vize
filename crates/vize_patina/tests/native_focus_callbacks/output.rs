use super::support::*;
use vize_patina::{HelpLevel, Locale, Rule, RuleCategory, RuleMeta, Severity};

#[test]
fn actual_concrete_metadata_and_original_eight_complete_outputs_are_preserved() {
    for (rule, name, description) in [
        (
            &vize_patina::rules::a11y::NoAutofocus as &dyn Rule,
            AUTO,
            "Disallow the use of the autofocus attribute",
        ),
        (
            &vize_patina::rules::a11y::NoAccessKey as &dyn Rule,
            KEY,
            "Disallow the use of the accesskey attribute",
        ),
    ] {
        let expected = RuleMeta {
            name,
            description,
            category: RuleCategory::Accessibility,
            fixable: false,
            default_severity: Severity::Warning,
        };
        assert_eq!(format!("{:#?}", rule.meta()), format!("{expected:#?}"));
    }
    for locale in LOCALES {
        for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
            for (source, rule, range) in [
                (
                    "<input type=\"text\" :autofocus=\"true\" />",
                    AUTO,
                    Some((19, 36)),
                ),
                ("<div :accesskey=\"'h'\">Content</div>", KEY, Some((5, 21))),
                ("<div :[accesskey]=\"shortcut\">Content</div>", KEY, None),
                (
                    "<input type=\"text\" :[autofocus]=\"enabled\" />",
                    AUTO,
                    None,
                ),
                ("<input type=\"text\" />", AUTO, None),
                ("<input type=\"text\" autofocus />", AUTO, Some((19, 28))),
                ("<div>Content</div>", KEY, None),
                ("<div accesskey=\"h\">Content</div>", KEY, Some((5, 18))),
            ] {
                let configured =
                    linter(false, locale, help).with_enabled_rules(Some(vec![rule.into()]));
                let findings = range
                    .map(|(start, end)| {
                        finding(
                            rule,
                            locale,
                            help,
                            vize_l0::Span::new(start, end),
                            Severity::Warning,
                        )
                    })
                    .into_iter()
                    .collect();
                pair(&configured, source, expected(findings));
            }
        }
    }
}

#[test]
fn prop_full_bind_modifiers_shorthand_and_opaque_expression_bytes_match_exact_names() {
    for locale in LOCALES {
        for (head, rule) in [
            (":autofocus", AUTO),
            (".autofocus", AUTO),
            ("v-bind:autofocus.prop", AUTO),
            (":autofocus.camel", AUTO),
            (".accesskey", KEY),
            ("v-bind:accesskey.camel", KEY),
        ] {
            for value in ["'界&amp;'", "not JavaScript !!!", "", "[] + )"] {
                let attribute = format!("{head}=\"{value}\"");
                let source = format!("界\r\n<div {attribute}></div>");
                pair(
                    &linter(false, locale, HelpLevel::Full),
                    &source,
                    expected(vec![finding(
                        rule,
                        locale,
                        HelpLevel::Full,
                        span(&source, &attribute),
                        Severity::Warning,
                    )]),
                );
            }
        }
        for (source, authored, rule) in [
            ("<input :autofocus />", ":autofocus", AUTO),
            ("<div v-bind:accesskey></div>", "v-bind:accesskey", KEY),
        ] {
            pair(
                &linter(false, locale, HelpLevel::Full),
                source,
                expected(vec![finding(
                    rule,
                    locale,
                    HelpLevel::Full,
                    span(source, authored),
                    Severity::Warning,
                )]),
            );
        }
    }
}

#[test]
fn case_sensitive_static_and_fixed_names_and_dynamic_runtime_target_strings_are_clean() {
    for source in [
        "<div autoFocus ACCESSKEY :autoFocus='opaque' v-bind:ACCESSKEY='opaque'></div>",
        "<div :[autofocus]='opaque' .[accesskey]='opaque'></div>",
        "<div :['autofocus']='opaque' v-bind:['accesskey']='opaque'></div>",
        "<div title='autofocus accesskey' data-autofocus data-accesskey></div>",
    ] {
        for locale in LOCALES {
            pair(
                &linter(false, locale, HelpLevel::Full),
                source,
                expected(vec![]),
            );
        }
    }
}

#[test]
fn actual_registry_rule_then_attribute_then_child_order_stays_unsorted_with_unicode_geometry() {
    let source = "界\r\n<div accesskey='parent' :autofocus='first' :accesskey='second' autofocus='last'><span accesskey='child' autofocus='child'/></div>";
    for reverse in [false, true] {
        for locale in LOCALES {
            let mut findings = Vec::new();
            for (auto, key) in [
                (
                    [":autofocus='first'", "autofocus='last'"].as_slice(),
                    ["accesskey='parent'", ":accesskey='second'"].as_slice(),
                ),
                (
                    ["autofocus='child'"].as_slice(),
                    ["accesskey='child'"].as_slice(),
                ),
            ] {
                let ordered = if reverse {
                    [(KEY, key), (AUTO, auto)]
                } else {
                    [(AUTO, auto), (KEY, key)]
                };
                for (rule, attrs) in ordered {
                    for authored in attrs {
                        findings.push(finding(
                            rule,
                            locale,
                            HelpLevel::Full,
                            span(source, authored),
                            Severity::Warning,
                        ));
                    }
                }
            }
            pair(
                &linter(reverse, locale, HelpLevel::Full),
                source,
                expected(findings),
            );
        }
    }
}
