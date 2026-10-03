use super::support::{LOCALES, error, expected, parity, span};

#[test]
fn exact_own_and_inherited_pre_keep_only_literal_exact_matching_attributes() {
    for source in [
        "<template><meta v-pre role :aria-hidden='x' .role='x' /></template>",
        "<template><div v-pre><meta role :aria-hidden='x' .role='x' /></div></template>",
        "<template><meta v-pre.foo v-pre role :aria-hidden='x' /></template>",
        "<template><meta v-pre v-pre.foo role :aria-hidden='x' /></template>",
    ] {
        for locale in LOCALES {
            assert_eq!(
                parity(source, locale),
                expected(vec![error(locale, span(source, "role"), "meta", "role")])
            );
        }
    }
}

#[test]
fn inherited_pre_retains_malformed_and_dynamic_heads_as_literal_original_attributes() {
    let source = "<template><div v-pre><meta role :title.='x' v-bind:aria-hidden..camel='x' v-unknown='x' :[key]='x' /></div></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![error(locale, span(source, "role"), "meta", "role")])
        );
    }
}

#[test]
fn nested_authentic_original_elements_keep_complete_ordered_results() {
    let source = "<template><div><meta role='first' /><html aria-hidden='last'><meta :role='second' /></html></div></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![
                error(locale, span(source, "role='first'"), "meta", "role"),
                error(
                    locale,
                    span(source, "aria-hidden='last'"),
                    "html",
                    "aria-hidden"
                ),
                error(locale, span(source, ":role='second'"), "meta", "role"),
            ])
        );
    }
}
