use super::support::{
    HeaderRule, LOCALES, NeverLookup, complete, expected, parity, parser, registered, selected,
    span, warning,
};
use vize_l0::{Allocator, cstr};
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn original_foster_notice_refuses_the_whole_header_before_any_lookup() {
    for tag in ["html", "marquee"] {
        let opening = cstr!("<{tag} autofocus accesskey='a'>");
        let source = cstr!("<template><table>{opening}</{tag}></table></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let before = cstr!("{:?}", owner.component().carrier());
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let table = owner.children().next().unwrap().into_element().unwrap();
        let element = table.children().next().unwrap().into_element().unwrap();
        let receipt = element.lint_tag().unwrap();
        assert!(receipt.in_table_context());
        for rule in HeaderRule::ALL {
            assert_eq!(
                rule.check(&lint, &element, &NeverLookup).err(),
                Some(NativeLintRefusal::TableContext {
                    span: receipt.span()
                })
            );
            for locale in LOCALES {
                let mut diagnostics = vec![parser(
                    "error",
                    "Foster parenting moved this element before the nearest open table.",
                    span(&source, &opening),
                )];
                if tag == "marquee" || !matches!(rule, HeaderRule::Distracting) {
                    let range = if matches!(rule, HeaderRule::Distracting) {
                        span(&source, &opening)
                    } else {
                        span(
                            &source,
                            &cstr!(
                                "{}{}",
                                rule.attribute(),
                                if matches!(rule, HeaderRule::AccessKey) {
                                    "='a'"
                                } else {
                                    ""
                                }
                            ),
                        )
                    };
                    diagnostics.push(warning(rule, locale, range, tag));
                }
                assert_eq!(
                    complete(&registered(&source, locale, rule)),
                    expected(diagnostics)
                );
            }
        }
        assert!(owner.component().carrier().errors.is_empty());
        assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
    }
}

#[test]
fn valid_table_descendants_keep_truthful_conservative_refusal() {
    let source = "<template><table><tbody><tr><td><input autofocus accesskey='a' /></td></tr></tbody></table></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let table = owner.children().next().unwrap().into_element().unwrap();
    let body = table.children().next().unwrap().into_element().unwrap();
    let row = body.children().next().unwrap().into_element().unwrap();
    let cell = row.children().next().unwrap().into_element().unwrap();
    let input = cell.children().next().unwrap().into_element().unwrap();
    for rule in HeaderRule::ALL {
        assert_eq!(
            rule.check(&lint, &input, &NeverLookup).err(),
            Some(NativeLintRefusal::TableContext {
                span: input.lint_tag().unwrap().span()
            })
        );
        for locale in LOCALES {
            let diagnostics = if matches!(rule, HeaderRule::Distracting) {
                vec![]
            } else {
                vec![warning(
                    rule,
                    locale,
                    span(
                        source,
                        &cstr!(
                            "{}{}",
                            rule.attribute(),
                            if matches!(rule, HeaderRule::AccessKey) {
                                "='a'"
                            } else {
                                ""
                            }
                        ),
                    ),
                    "input",
                )]
            };
            assert_eq!(
                complete(&registered(source, locale, rule)),
                expected(diagnostics)
            );
        }
    }
}

#[test]
fn own_table_headers_and_following_siblings_remain_independently_admitted() {
    let source = "<template><table autofocus accesskey='a'></table><input autofocus accesskey='b' /></template>";
    for rule in HeaderRule::ALL {
        for locale in LOCALES {
            parity(source, locale, rule);
        }
    }
}
