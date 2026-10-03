use super::support::{LOCALES, expected, parity, span, warning};
use vize_l0::cstr;

#[test]
fn all_original_product_spellings_keep_complete_registered_output() {
    for tag in [
        "acronym",
        "applet",
        "basefont",
        "bgsound",
        "big",
        "blink",
        "center",
        "dir",
        "font",
        "frame",
        "frameset",
        "isindex",
        "keygen",
        "listing",
        "marquee",
        "menuitem",
        "multicol",
        "nextid",
        "nobr",
        "noembed",
        "noframes",
        "plaintext",
        "rb",
        "rtc",
        "spacer",
        "strike",
        "tt",
        "xmp",
    ] {
        for self_closing in [false, true] {
            let opening = if self_closing {
                cstr!("<{tag} data-label='名' />")
            } else {
                cstr!("<{tag} data-label='名'>")
            };
            let closing = if self_closing {
                cstr!("")
            } else {
                cstr!("文字</{tag}>")
            };
            let source = cstr!("<template>{opening}{closing}</template>");
            for locale in LOCALES {
                assert_eq!(
                    parity(&source, locale),
                    expected(vec![warning(locale, span(&source, &opening), tag)]),
                    "{tag}: {self_closing}"
                );
            }
        }
    }
}

#[test]
fn component_casing_builtins_unknown_and_qualified_lookalikes_stay_clean() {
    for tag in [
        "div",
        "widget",
        "my-center",
        "center-x",
        "x:center",
        "foo:font",
        "aÉCenter",
        "Center",
        "CENTER",
        "Font",
        "AÉCenter",
        "Teleport",
        "Suspense",
        "KeepAlive",
        "BaseTransition",
        "Transition",
        "TransitionGroup",
        "teleport",
        "transition-group",
        "slot",
        "template",
        "component",
    ] {
        let source = cstr!("<template><{tag} is='center' :is='view'></{tag}></template>");
        for locale in LOCALES {
            assert_eq!(parity(&source, locale), expected(vec![]), "{tag}");
        }
    }
}

#[test]
fn opaque_controls_and_values_do_not_grant_runtime_component_or_expression_meaning() {
    for control in [
        "is='Foo' :is='view'",
        ":title.prop.camel='child(' .id='x'",
        "@click.stop='value' v-show='unknown(' v-cloak",
        "data-label='名&amp;前' title=unquoted",
    ] {
        let opening = cstr!("<center {control}>");
        let source = cstr!("<template>{opening}本文</center></template>");
        for locale in LOCALES {
            assert_eq!(
                parity(&source, locale),
                expected(vec![warning(locale, span(&source, &opening), "center")])
            );
        }
    }
}

#[test]
fn namespace_context_keeps_authored_tag_grammar_and_full_original_ranges() {
    let source = "<template><svg><foreignObject><center /></foreignObject><font /></svg><math><mi><blink /></mi><annotation-xml encoding='text/html'><marquee /></annotation-xml></math></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![
                warning(locale, span(source, "<center />"), "center"),
                warning(locale, span(source, "<font />"), "font"),
                warning(locale, span(source, "<blink />"), "blink"),
                warning(locale, span(source, "<marquee />"), "marquee"),
            ])
        );
    }
}

#[test]
fn nested_original_openings_keep_absolute_unicode_offsets_and_complete_catalog_help() {
    let source = "<!--🦀--><script>const fake='<font />'</script><template>日本語<center title='親'><widget><blink\n data-label=\"名&amp;前\" /></widget></center></template>";
    for locale in LOCALES {
        let result = parity(source, locale);
        assert_eq!(
            result,
            expected(vec![
                warning(locale, span(source, "<center title='親'>"), "center"),
                warning(
                    locale,
                    span(source, "<blink\n data-label=\"名&amp;前\" />"),
                    "blink"
                ),
            ])
        );
    }
}

#[test]
fn actual_raw_text_children_keep_fake_deprecated_markup_as_original_text() {
    let source = "<template><script>const fake='<center />'</script><style>.x { color:red } /* <font /> */</style><title>名 &amp; <blink /></title><textarea>本文 <marquee /></textarea><xmp>文字</xmp></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![warning(locale, span(source, "<xmp>"), "xmp")])
        );
    }
}
