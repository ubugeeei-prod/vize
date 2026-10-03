use super::support::{LOCALES, error, expected, parity, span};
use vize_l0::cstr;

#[test]
fn every_exact_static_bind_and_prop_match_has_complete_ordered_localized_error_output() {
    let source = "<script>const π = 1</script>\n<template><meta aria-label=\"字&amp;\" role :aria-hidden.camel='opaque' v-bind:role.prop='value' .aria-x='x' @role='handler' /></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![
                error(
                    locale,
                    span(source, "aria-label=\"字&amp;\""),
                    "meta",
                    "aria-label"
                ),
                error(locale, span(source, "role"), "meta", "role"),
                error(
                    locale,
                    span(source, ":aria-hidden.camel='opaque'"),
                    "meta",
                    "aria-hidden"
                ),
                error(
                    locale,
                    span(source, "v-bind:role.prop='value'"),
                    "meta",
                    "role"
                ),
                error(locale, span(source, ".aria-x='x'"), "meta", "aria-x"),
            ])
        );
    }
}

#[test]
fn all_four_original_lowercase_tags_and_raw_text_children_keep_complete_registered_output() {
    let source = "<template><meta aria-hidden /><html role='document'></html><script :role='x'>const text = '<meta role />';</script><style v-bind:aria-label='x'>/* <meta role /> */</style></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![
                error(locale, span(source, "aria-hidden"), "meta", "aria-hidden"),
                error(locale, span(source, "role='document'"), "html", "role"),
                error(locale, span(source, ":role='x'"), "script", "role"),
                error(
                    locale,
                    span(source, "v-bind:aria-label='x'"),
                    "style",
                    "aria-label"
                ),
            ])
        );
    }
}

#[test]
fn uppercase_components_custom_names_and_exact_uppercase_attribute_controls_are_exempt() {
    for tag in [
        "Meta",
        "META",
        "HTML",
        "Script",
        "STYLE",
        "Foo",
        "KeepAlive",
        "div",
        "x-meta",
        "meta:part",
        "slot",
        "template",
    ] {
        let source = cstr!("<template><{tag} role aria-hidden :role='x' /></template>");
        for locale in LOCALES {
            assert_eq!(parity(&source, locale), expected(vec![]));
        }
    }
    for source in [
        "<template><meta ROLE ARIA-hidden :ROLE='x' /></template>",
        "<template><meta @role='x' v-model:role='x' title='ok' /></template>",
    ] {
        for locale in LOCALES {
            assert_eq!(parity(source, locale), expected(vec![]));
        }
    }
}

#[test]
fn empty_prefix_attributes_and_opaque_values_are_preserved_exactly() {
    let source =
        "<template><meta aria-='{{not-an-expression}}' :aria-='malformed(' role='' /></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![
                error(
                    locale,
                    span(source, "aria-='{{not-an-expression}}'"),
                    "meta",
                    "aria-"
                ),
                error(locale, span(source, ":aria-='malformed('"), "meta", "aria-"),
                error(locale, span(source, "role=''"), "meta", "role"),
            ])
        );
    }
}
