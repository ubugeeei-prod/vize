use super::support::{LOCALES, expected, parity, span, warning};
use vize_l0::cstr;

const ALIGN: &str = "CSS `text-align` or `margin: auto`";

#[test]
fn static_forms_retain_entire_absolute_authored_attributes() {
    for header in [
        "align",
        "align=''",
        "align=\"\"",
        "align=center",
        "align = '名&amp;前'",
        "align=\"one;\n two\"",
    ] {
        let source = cstr!("<!--🦀--><template>日本語<div {header} /></template>");
        for locale in LOCALES {
            assert_eq!(
                parity(&source, locale),
                expected(vec![warning(
                    locale,
                    span(&source, header),
                    "div",
                    "align",
                    ALIGN
                )])
            );
        }
    }
}

#[test]
fn static_values_are_opaque_even_when_empty_entities_or_invalid_expressions() {
    for value in [
        "",
        " ",
        "false",
        "undefined",
        "broken(",
        "a +",
        "名",
        "&#0;",
        "&#x41;&amp;",
    ] {
        let header = cstr!("align='{value}'");
        let source = cstr!("<template><div {header} /></template>");
        for locale in LOCALES {
            assert_eq!(
                parity(&source, locale),
                expected(vec![warning(
                    locale,
                    span(&source, &header),
                    "div",
                    "align",
                    ALIGN
                )])
            );
        }
    }
}

#[test]
fn typed_bind_prop_and_modifiers_are_nonmatching_without_value_parsing() {
    for head in [
        ":align",
        ".align",
        "v-bind:align",
        ":align.prop.camel.sync.arbitrary",
    ] {
        for tail in ["", "='broken('", "='&#0;'"] {
            let source = cstr!("<template><div {head}{tail} /></template>");
            for locale in LOCALES {
                assert_eq!(parity(&source, locale), expected(vec![]));
            }
        }
    }
}

#[test]
fn case_qualified_attribute_and_nonbinding_directives_never_match_static_policy() {
    for head in [
        "ALIGN",
        "Align",
        "data-align",
        "foo:align",
        "align.foo",
        "@align",
        "v-on:align.once",
        "v-model:align",
    ] {
        let source = cstr!("<template><div {head}='broken(' /></template>");
        for locale in LOCALES {
            assert_eq!(parity(&source, locale), expected(vec![]));
        }
    }
}

#[test]
fn actual_uppercase_components_are_exempt() {
    for source in [
        "<template><Foo align bgcolor border /></template>",
        "<template><Component align /></template>",
        "<template><Transition align /></template>",
    ] {
        for locale in LOCALES {
            assert_eq!(parity(source, locale), expected(vec![]));
        }
    }
}

#[test]
fn lowercase_custom_and_foreign_namespace_elements_use_actual_lint_categories() {
    let source = "<template><foo-box align='one' /><svg><path align='two' /></svg><math><mi align='three' /></math></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(
                [
                    ("foo-box", "align='one'"),
                    ("path", "align='two'"),
                    ("mi", "align='three'")
                ]
                .iter()
                .map(|(tag, head)| warning(locale, span(source, head), tag, "align", ALIGN))
                .collect()
            )
        );
    }
}

#[test]
fn multiple_distinct_findings_keep_authored_not_policy_order() {
    let source = "<!--🦀--><template><body alink='first' bgcolor='second' text='third' align='fourth' background='fifth' /></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(
                [
                    ("alink='first'", "alink", "CSS `:link`, `:visited`"),
                    ("bgcolor='second'", "bgcolor", "CSS `background-color`"),
                    ("text='third'", "text", "CSS `color`"),
                    ("align='fourth'", "align", ALIGN),
                    ("background='fifth'", "background", "CSS `background-image`"),
                ]
                .iter()
                .map(|(head, attr, help)| warning(locale, span(source, head), "body", attr, help))
                .collect()
            )
        );
    }
}

#[test]
fn raw_element_bodies_never_create_fake_matching_headers() {
    let source = "<template><script align='a'><div align='fake' /></script><style align='b'><div align='fake' /></style><textarea align='c'>&lt;div align='fake'&gt;</textarea><title align='d'>&lt;div align='fake'&gt;</title></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(
                [
                    ("script", "align='a'"),
                    ("style", "align='b'"),
                    ("textarea", "align='c'"),
                    ("title", "align='d'")
                ]
                .iter()
                .map(|(tag, head)| warning(locale, span(source, head), tag, "align", ALIGN))
                .collect()
            )
        );
    }
}

#[test]
fn lowercase_component_slot_and_structural_template_are_not_component_exemptions() {
    for (tag, source) in [
        (
            "component",
            "<template><component :is='broken(' align /></template>",
        ),
        ("slot", "<template><slot align /></template>"),
        (
            "template",
            "<template><template v-if='ok' align></template></template>",
        ),
    ] {
        for locale in LOCALES {
            assert_eq!(
                parity(source, locale),
                expected(vec![warning(
                    locale,
                    span(source, "align"),
                    tag,
                    "align",
                    ALIGN
                )])
            );
        }
    }
}
