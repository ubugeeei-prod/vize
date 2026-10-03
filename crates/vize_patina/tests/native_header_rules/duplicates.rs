use super::support::{
    HeaderRule, LOCALES, NeverLookup, complete, expected, parser, registered, selected, span,
    warning,
};
use vize_l0::{Allocator, cstr};
use vize_patina::Locale;
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn duplicate_unrelated_static_attributes_refuse_even_when_components_are_exempt() {
    for tag in ["marquee", "Foo"] {
        let opening = cstr!("<{tag} autofocus accesskey id='one' ID='two'>");
        let source = cstr!("<template>{opening}</{tag}></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let repeated = span(&source, "ID");
        let duplicate = parser(
            "warning",
            "Duplicate attribute `ID`. Keeping the repeated attribute so parsing can continue.",
            repeated,
        );
        for rule in HeaderRule::ALL {
            assert_eq!(
                rule.check(&lint, &element, &NeverLookup).err(),
                Some(NativeLintRefusal::DuplicateAttribute { span: repeated })
            );
            for locale in LOCALES {
                let mut diagnostics = vec![duplicate.clone()];
                if tag != "Foo" {
                    let finding_span = if matches!(rule, HeaderRule::Distracting) {
                        span(&source, &opening)
                    } else {
                        span(&source, rule.attribute())
                    };
                    diagnostics.push(warning(rule, locale, finding_span, tag));
                }
                assert_eq!(
                    complete(&registered(&source, locale, rule)),
                    expected(diagnostics)
                );
            }
        }
        assert!(owner.component().carrier().errors.is_empty());
    }
}

#[test]
fn duplicate_target_case_keeps_original_findings_and_parser_notice_in_the_oracle() {
    for rule in HeaderRule::ATTRIBUTES {
        let name = rule.attribute();
        let repeated = name.to_ascii_uppercase();
        let header = cstr!("{name}='one' {repeated}='two'");
        let source = cstr!("<template><input {header} /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let duplicate_span = span(&source, &repeated);
        assert_eq!(
            rule.check(&lint, &element, &NeverLookup).err(),
            Some(NativeLintRefusal::DuplicateAttribute {
                span: duplicate_span
            })
        );
        for locale in LOCALES {
            let message = cstr!(
                "Duplicate attribute `{repeated}`. Keeping the repeated attribute so parsing can continue."
            );
            assert_eq!(
                complete(&registered(&source, locale, rule)),
                expected(vec![
                    warning(rule, locale, span(&source, &cstr!("{name}='one'")), "input"),
                    parser("warning", &message, duplicate_span)
                ])
            );
        }
    }
}

#[test]
fn inherited_pre_duplicate_raw_directive_spellings_keep_the_parser_notice() {
    let source = "<template><div v-pre><marquee autofocus accesskey :title='one' :TITLE='two'></marquee></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let element = parent.children().next().unwrap().into_element().unwrap();
    let repeated = span(source, ":TITLE");
    for rule in HeaderRule::ALL {
        assert_eq!(
            rule.check(&lint, &element, &NeverLookup).err(),
            Some(NativeLintRefusal::DuplicateAttribute { span: repeated })
        );
        let finding_span = if matches!(rule, HeaderRule::Distracting) {
            span(
                source,
                "<marquee autofocus accesskey :title='one' :TITLE='two'>",
            )
        } else {
            span(source, rule.attribute())
        };
        assert_eq!(
            complete(&registered(source, Locale::En, rule)),
            expected(vec![
                warning(rule, Locale::En, finding_span, "marquee"),
                parser(
                    "warning",
                    "Duplicate attribute `:TITLE`. Keeping the repeated attribute so parsing can continue.",
                    repeated
                )
            ])
        );
    }
}
