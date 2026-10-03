use super::support::{LOCALES, expected, parity, span, warning};
use vize_l0::cstr;

#[test]
fn static_attributes_retain_complete_authored_ranges_in_every_locale() {
    for header in [
        "style",
        "style=''",
        "style=\"\"",
        "style=color:red",
        "style = '名&amp;前'",
        "style=\"color: red;\nmargin: 0\"",
    ] {
        let source = cstr!("<!--🦀--><template>日本語<div {header} /></template>");
        for locale in LOCALES {
            assert_eq!(
                parity(&source, locale),
                expected(vec![warning(locale, span(&source, header))])
            );
        }
    }
}

#[test]
fn values_are_opaque_and_static_presence_alone_warns() {
    for value in [
        "",
        " ",
        "false",
        "undefined",
        "broken(",
        "a +",
        "名",
        "&#0;",
    ] {
        let header = cstr!("style='{value}'");
        let source = cstr!("<template><div {header} /></template>");
        for locale in LOCALES {
            assert_eq!(
                parity(&source, locale),
                expected(vec![warning(locale, span(&source, &header))])
            );
        }
    }
}

#[test]
fn typed_bind_and_prop_style_heads_remain_clean_without_value_parsing() {
    for head in [
        ":style",
        ".style",
        "v-bind:style",
        ":style.prop.camel.sync.arbitrary",
    ] {
        for tail in ["", "='broken('", "='&#0;'"] {
            let source = cstr!("<template><Foo {head}{tail} /></template>");
            for locale in LOCALES {
                assert_eq!(parity(&source, locale), expected(vec![]));
            }
        }
    }
}

#[test]
fn components_builtin_slots_templates_and_namespaces_are_not_exempt() {
    let source = "<template><Foo style='one' :style='broken(' /><component :is='broken(' style /><slot style='two' /><template v-if='ok' style='three'></template><svg><path style='four' /></svg><math><mi style='five' /></math></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(
                [
                    "style='one'",
                    "style />",
                    "style='two'",
                    "style='three'",
                    "style='four'",
                    "style='five'"
                ]
                .iter()
                .map(|spelling| {
                    let mut range = span(source, spelling);
                    if *spelling == "style />" {
                        range.end = range.start + 5;
                    }
                    warning(locale, range)
                })
                .collect()
            )
        );
    }
}

#[test]
fn case_namespace_and_nonbinding_directives_never_become_style_attributes() {
    for head in [
        "STYLE",
        "Style",
        "data-style",
        "foo:style",
        "style.foo",
        "@style",
        "v-on:style.once",
        "v-model:style",
    ] {
        let source = cstr!("<template><div {head}='broken(' /></template>");
        for locale in LOCALES {
            assert_eq!(parity(&source, locale), expected(vec![]));
        }
    }
}

#[test]
fn raw_element_text_is_not_walked_as_fake_style_headers() {
    let source = "<template><script style='a'><Foo style='fake' /></script><style style='b'><div style='fake' /></style><textarea style='c'>&lt;div style='fake'&gt;</textarea><title style='d'>&lt;div style='fake'&gt;</title></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(
                ["style='a'", "style='b'", "style='c'", "style='d'"]
                    .iter()
                    .map(|spelling| warning(locale, span(source, spelling)))
                    .collect()
            )
        );
    }
}
