use super::support::{HeaderRule, LOCALES, expected, native, parity, selected, span, warning};
use vize_carton::i18n::{Locale, translator};
use vize_l0::{Allocator, cstr};
use vize_patina::native::NativeSyntaxLint;

#[test]
fn exact_static_bind_and_prop_names_keep_complete_attribute_ranges_in_every_locale() {
    for rule in HeaderRule::ATTRIBUTES {
        let name = rule.attribute();
        for header in [
            name.to_owned(),
            format!("{name}=''"),
            format!("{name}=false"),
            format!("{name}=\"名&amp;前\""),
            format!(":{name}='false'"),
            format!(".{name}='broken('"),
            format!("v-bind:{name}=\"value\""),
            format!(":{name}"),
            format!(".{name}"),
            format!("v-bind:{name}"),
            format!(":{name}.prop.camel.sync.arbitrary='value'"),
            format!(".{name}.camel='value'"),
        ] {
            let source = cstr!("<!--🦀--><template>日本語<input {header} /></template>");
            for locale in LOCALES {
                let output = parity(&source, locale, rule);
                assert_eq!(
                    output,
                    expected(vec![warning(rule, locale, span(&source, &header), "input")]),
                    "{header}"
                );
            }
        }
    }
}

#[test]
fn attribute_values_remain_opaque_even_when_expression_parsing_cannot_finish() {
    for rule in HeaderRule::ATTRIBUTES {
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
            for prefix in ["", ":", ".", "v-bind:"] {
                let header = cstr!("{prefix}{}='{value}'", rule.attribute());
                let source = cstr!("<template><input {header} /></template>");
                assert_eq!(parity(&source, Locale::En, rule)["warning_count"], 1);
            }
        }
    }
}

#[test]
fn case_lookalikes_qualified_arguments_and_nonbinding_directives_do_not_match() {
    for rule in HeaderRule::ATTRIBUTES {
        let name = rule.attribute();
        let camel = match rule {
            HeaderRule::Autofocus => "autoFocus",
            _ => "accessKey",
        };
        let hyphen = match rule {
            HeaderRule::Autofocus => "auto-focus",
            _ => "access-key",
        };
        for header in [
            format!("{}='x'", name.to_ascii_uppercase()),
            format!("{camel}='x'"),
            format!("data-{name}='x'"),
            format!("foo:{name}='x'"),
            format!("{name}.foo='x'"),
            format!(":{camel}='x'"),
            format!(":{hyphen}.camel='x'"),
            format!(":{name}:extra='x'"),
            format!("v-bind:{name}:extra='x'"),
            format!("@{name}='handler'"),
            format!("v-on:{name}.once='handler'"),
            format!("v-model:{name}='model'"),
        ] {
            let source = cstr!("<template><input {header} /></template>");
            assert_eq!(
                parity(&source, Locale::En, rule),
                expected(vec![]),
                "{header}"
            );
        }
    }
}

#[test]
fn multiple_authored_binding_spellings_keep_return_order_before_product_sorting() {
    for rule in HeaderRule::ATTRIBUTES {
        let name = rule.attribute();
        let spellings = [
            cstr!(".{name}='first'"),
            cstr!("{name}='second'"),
            cstr!("v-bind:{name}.prop='third'"),
            cstr!(":{name}.camel='fourth'"),
        ];
        let source = cstr!(
            "<template><input {} {} {} {} /></template>",
            spellings[0],
            spellings[1],
            spellings[2],
            spellings[3]
        );
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        for locale in LOCALES {
            let findings = rule
                .check(&lint, &element, &translator().for_locale(locale))
                .unwrap();
            assert_eq!(findings.len(), spellings.len());
            for (finding, spelling) in findings.iter().zip(&spellings) {
                assert_eq!(
                    native(finding),
                    warning(rule, locale, span(&source, spelling), "input")
                );
            }
            assert_eq!(parity(&source, locale, rule)["warning_count"], 4);
        }
    }
}

#[test]
fn nonvoid_self_closing_compatibility_notice_is_suppressed_by_the_actual_entry() {
    let source = "<template><div autofocus accesskey /></template>";
    for rule in HeaderRule::ALL {
        for locale in LOCALES {
            let diagnostics = if matches!(rule, HeaderRule::Distracting) {
                vec![]
            } else {
                vec![warning(rule, locale, span(source, rule.attribute()), "div")]
            };
            assert_eq!(parity(source, locale, rule), expected(diagnostics));
        }
    }
}
