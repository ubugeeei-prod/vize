use super::support::{HeaderRule, LOCALES, expected, parity, span, warning};
use vize_l0::cstr;
use vize_patina::Locale;

#[test]
fn original_intrinsic_categories_keep_components_custom_tags_and_opaque_is_controls() {
    for (tag, control, component) in [
        ("div", "", false),
        ("widget", "", false),
        ("my-element", "", false),
        ("marquee", "", false),
        ("blink", "", false),
        ("marquee-x", "", false),
        ("x:blink", "", false),
        ("foo:bar", "", false),
        ("aÉFoo", "", false),
        ("teleport", "", false),
        ("transition-group", "", false),
        ("component", ":is='view'", false),
        ("input", "is='Foo' :is='view'", false),
        ("Foo", "is='input'", true),
        ("INPUT", "", true),
        ("AÉFoo", "", true),
        ("Teleport", "", true),
        ("Suspense", "", true),
        ("KeepAlive", "", true),
        ("BaseTransition", "", true),
        ("Transition", "", true),
        ("TransitionGroup", "", true),
        ("Marquee", "", true),
        ("BLINK", "", true),
        ("slot", "name='default'", false),
        ("template", "v-if='ok'", false),
        ("template", "v-slot:default", false),
        ("template", "if='lookalike'", false),
    ] {
        let source = if tag == "input" {
            cstr!("<template><{tag} {control} autofocus accesskey /></template>")
        } else {
            cstr!("<template><{tag} {control} autofocus accesskey></{tag}></template>")
        };
        for rule in HeaderRule::ALL {
            for locale in LOCALES {
                let result = parity(&source, locale, rule);
                let warnings = usize::from(
                    !component
                        && (!matches!(rule, HeaderRule::Distracting)
                            || matches!(tag, "marquee" | "blink")),
                );
                assert_eq!(result["warning_count"], warnings, "{tag}: {rule:?}");
            }
        }
    }
}

#[test]
fn native_namespaces_do_not_change_authored_rule_predicates() {
    let source = "<template><svg autofocus accesskey><foreignObject autofocus accesskey><widget autofocus accesskey /></foreignObject><blink autofocus accesskey /></svg><math autofocus accesskey><mi autofocus accesskey /><annotation-xml encoding='text/html'><marquee autofocus accesskey /></annotation-xml></math></template>";
    for rule in HeaderRule::ALL {
        for locale in LOCALES {
            let result = parity(source, locale, rule);
            assert_eq!(
                result["warning_count"],
                if matches!(rule, HeaderRule::Distracting) {
                    2
                } else {
                    8
                }
            );
        }
    }
}

#[test]
fn distracting_elements_keep_whole_opening_ranges_and_full_catalog_help() {
    for opening in [
        "<marquee title='名'>",
        "<blink\n data-label=\"名&amp;前\" />",
    ] {
        let tag = if opening.starts_with("<marquee") {
            "marquee"
        } else {
            "blink"
        };
        let closing = if tag == "marquee" {
            "本文<span>nested</span></marquee>"
        } else {
            ""
        };
        let source = cstr!(
            "<!--🦀--><script>const fake='<blink />'</script><template>日本語{opening}{closing}</template>"
        );
        for locale in LOCALES {
            let result = parity(&source, locale, HeaderRule::Distracting);
            assert_eq!(
                result,
                expected(vec![warning(
                    HeaderRule::Distracting,
                    locale,
                    span(&source, opening),
                    tag
                )])
            );
            if locale == Locale::Zh {
                let help = result["diagnostics"][0]["help"].as_str().unwrap();
                assert!(help.contains("```css"));
                assert!(help.contains("prefers-reduced-motion"));
            }
        }
    }
}

#[test]
fn nested_unicode_and_original_attribute_order_keep_absolute_sfc_byte_offsets() {
    let source = "<!--🦀--><script>const fake='<blink autofocus accesskey />'</script><template>日本語<marquee autofocus='親' accesskey='親'><widget :autofocus.prop='child(' .accesskey='x'><blink autofocus accesskey /></widget></marquee></template>";
    for rule in HeaderRule::ALL {
        let output = parity(source, Locale::Ja, rule);
        let slices: Vec<_> = output["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .map(|finding| {
                let start = finding["start"].as_u64().unwrap() as usize;
                let end = finding["end"].as_u64().unwrap() as usize;
                &source[start..end]
            })
            .collect();
        assert_eq!(
            slices,
            match rule {
                HeaderRule::Autofocus =>
                    vec!["autofocus='親'", ":autofocus.prop='child('", "autofocus"],
                HeaderRule::AccessKey => vec!["accesskey='親'", ".accesskey='x'", "accesskey"],
                HeaderRule::Distracting => vec![
                    "<marquee autofocus='親' accesskey='親'>",
                    "<blink autofocus accesskey />"
                ],
            }
        );
    }
}
