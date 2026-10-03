use super::support::{LOCALES, expected, parity, span, warning};
use vize_l0::{Span, cstr};
use vize_patina::{Rule, RuleCategory, Severity, rules::vue::NoVHtml};

#[test]
fn registered_warning_essential_nonfixable_metadata_is_retained() {
    let rule = NoVHtml;
    assert_eq!(rule.meta().name, "vue/no-v-html");
    assert_eq!(rule.meta().category, RuleCategory::Essential);
    assert_eq!(rule.meta().default_severity, Severity::Warning);
    assert!(!rule.meta().fixable);
}

#[test]
fn full_directives_keep_entire_authored_values_arguments_and_modifiers() {
    for head in [
        "v-html",
        "v-html.foo",
        "v-html:arg",
        "v-html:arg.camel",
        "v-html.ユーザー",
    ] {
        for value in [
            "",
            "='x'",
            "=plain",
            "=\"&lt;invalid(&gt;\"",
            "=''",
            "='   '",
        ] {
            let attribute = cstr!("{head}{value}");
            let source = cstr!("<template><div {attribute} /></template>");
            for locale in LOCALES {
                assert_eq!(
                    parity(&source, locale),
                    expected(vec![warning(locale, span(&source, &attribute))])
                );
            }
        }
    }
}

#[test]
fn static_bind_and_prop_arguments_keep_name_only_ranges() {
    for sink in ["innerHTML", "outerHTML"] {
        for prefix in [":", ".", "v-bind:"] {
            for modifier in ["", ".prop", ".camel.sync"] {
                for value in ["", "='broken('", "=''", "=plain", "='&lt;opaque&gt;'"] {
                    let source =
                        cstr!("<template><div {prefix}{sink}{modifier}{value} /></template>");
                    for locale in LOCALES {
                        assert_eq!(
                            parity(&source, locale),
                            expected(vec![warning(locale, span(&source, sink))])
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn components_custom_namespaces_and_foreign_tags_have_no_exemption() {
    for tag in [
        "Foo",
        "foo-bar",
        "foo",
        "div",
        "svg",
        "math",
        "g",
        "math:mi",
        "slot",
        "template",
        "component",
    ] {
        let source = cstr!("<template><{tag} v-html='x' :innerHTML='y' /></template>");
        for locale in LOCALES {
            assert_eq!(
                parity(&source, locale),
                expected(vec![
                    warning(locale, span(&source, "v-html='x'")),
                    warning(locale, span(&source, "innerHTML")),
                ])
            );
        }
    }
}

#[test]
fn repeated_directives_retain_full_authored_callback_order() {
    let source = "<template><div v-html='first' :outerHTML='x' v-html.foo='second' .innerHTML='y' v-html='last' /></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![
                warning(locale, span(source, "v-html='first'")),
                warning(locale, span(source, "outerHTML")),
                warning(locale, span(source, "v-html.foo='second'")),
                warning(locale, span(source, "innerHTML")),
                warning(locale, span(source, "v-html='last'")),
            ])
        );
    }
}

#[test]
fn exact_argument_case_static_lookalikes_and_other_directives_are_clean() {
    for head in [
        "innerHTML",
        "outerHTML",
        ":innerhtml",
        ".outerHtml",
        "v-bind:InnerHTML",
        "@innerHTML",
        "#innerHTML",
        "v-text",
        "v-show",
        "v-on:html",
    ] {
        let source = cstr!("<template><div {head}='opaque(' /></template>");
        for locale in LOCALES {
            assert_eq!(parity(&source, locale), expected(vec![]));
        }
    }
}

#[test]
fn static_bound_unrelated_values_do_not_search_object_sink_names() {
    let source = "<template><div :title='{ innerHTML: html }' v-bind:class='{ outerHTML: html }' .id='innerHTML' innerHTML='x' /></template>";
    for locale in LOCALES {
        assert_eq!(parity(source, locale), expected(vec![]));
    }
}

#[test]
fn absolute_unicode_offsets_and_multiple_elements_keep_original_order() {
    let source = "<!--🦀--><script>const x='光'</script><template>雪<div v-html='🙂' /><Foo :innerHTML='日本語' /><svg .outerHTML='x' /></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![
                warning(locale, span(source, "v-html='🙂'")),
                warning(locale, span(source, "innerHTML")),
                warning(locale, span(source, "outerHTML")),
            ])
        );
    }
    let authored = span(source, "v-html='🙂'");
    assert_eq!(authored, Span::new(59, 72));
}

#[test]
fn multiline_original_header_ranges_retain_every_byte() {
    let source = "<template><div\n class='x'\n v-html = \"opaque(\"\n :innerHTML .prop = 'x'\n /></template>";
    // A separated modifier is another binding, outside the original sink argument.
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![
                warning(locale, span(source, "v-html = \"opaque(\"")),
                warning(locale, span(source, "innerHTML")),
            ])
        );
    }
}

#[test]
fn authored_replacement_directives_leave_html_sink_callbacks_intact() {
    for head in ["v-if", "v-for", "v-text", "v-once", "v-memo", "#default"] {
        let value = if head == "v-for" {
            "item in items"
        } else {
            "ok"
        };
        let source =
            cstr!("<template><div {head}='{value}' v-html='html' :innerHTML='x' /></template>");
        for locale in LOCALES {
            assert_eq!(
                parity(&source, locale),
                expected(vec![
                    warning(locale, span(&source, "v-html='html'")),
                    warning(locale, span(&source, "innerHTML")),
                ])
            );
        }
    }
}
