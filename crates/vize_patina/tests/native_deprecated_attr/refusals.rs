use super::support::{
    LOCALES, NeverLookup, complete, expected, parity, parser, registered, selected, span, warning,
};
use vize_l0::{Allocator, Span, cstr};
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

const ALIGN: &str = "CSS `text-align` or `margin: auto`";

#[test]
fn late_unsupported_heads_refuse_matching_exempt_and_nonmatching_whole_results() {
    for (tag, attr, warns) in [
        ("div", "align", true),
        ("Foo", "align", false),
        ("div", "background", false),
    ] {
        for (head, unresolved) in [
            (":[key]", true),
            (".[key]", true),
            ("@[key]", true),
            ("v-bind:[key]", true),
            ("v-bind", true),
            ("v-bind.prop", true),
            ("v-unknown", false),
            (":title.", false),
        ] {
            let header = cstr!("{attr}='yes'");
            let source = cstr!("<template><{tag} {header} {head}='broken(' /></template>");
            let arena = Allocator::default();
            let owner = selected(&arena, &source);
            let lint = NativeSyntaxLint::new(&owner).unwrap();
            let element = owner.children().next().unwrap().into_element().unwrap();
            let range = span(&source, head);
            let refusal = if unresolved {
                NativeLintRefusal::UnresolvedBinding { span: range }
            } else {
                NativeLintRefusal::UnsupportedDirective { span: range }
            };
            assert_eq!(
                lint.deprecated_attr(&element, &NeverLookup).err(),
                Some(refusal)
            );
            for locale in LOCALES {
                let mut diagnostics = Vec::new();
                if warns {
                    diagnostics.push(warning(locale, span(&source, &header), tag, attr, ALIGN));
                }
                if head == ":title." {
                    diagnostics.push(parser(
                        "error",
                        "Directive modifier is expected.",
                        Span::new(range.end, range.end + 1),
                    ));
                }
                assert_eq!(
                    complete(&registered(&source, locale)),
                    expected(diagnostics)
                );
            }
        }
    }
}

#[test]
fn duplicates_refuse_before_lookup_and_keep_unsorted_complete_parser_product_order() {
    for (header, repeated, second_warns) in [
        ("align='one' align='two'", "align", true),
        ("align='one' ALIGN='two'", "ALIGN", false),
        ("align='one' id='one' ID='two'", "ID", false),
    ] {
        let source = cstr!("<template><div {header} /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let start = u32::try_from(source.rfind(repeated).unwrap()).unwrap();
        let range = Span::new(start, start + u32::try_from(repeated.len()).unwrap());
        assert_eq!(
            lint.deprecated_attr(&element, &NeverLookup).err(),
            Some(NativeLintRefusal::DuplicateAttribute { span: range })
        );
        let message = cstr!(
            "Duplicate attribute `{repeated}`. Keeping the repeated attribute so parsing can continue."
        );
        for locale in LOCALES {
            let mut diagnostics = vec![
                warning(locale, span(&source, "align='one'"), "div", "align", ALIGN),
                parser("warning", &message, range),
            ];
            if second_warns {
                diagnostics.push(warning(
                    locale,
                    span(&source, "align='two'"),
                    "div",
                    "align",
                    ALIGN,
                ));
            }
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(diagnostics)
            );
        }
    }
}

#[test]
fn fostered_and_valid_table_descendants_refuse_without_dropping_registered_findings() {
    for (source, path, tag, head, suggestion, foster) in [
        (
            "<template><table><div align='yes'></div></table></template>",
            &[0, 0][..],
            "div",
            "align='yes'",
            ALIGN,
            true,
        ),
        (
            "<template><table><tbody><tr><td width='yes'></td></tr></tbody></table></template>",
            &[0, 0, 0, 0][..],
            "td",
            "width='yes'",
            "CSS `width`/`height`",
            false,
        ),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let mut element = owner
            .children()
            .nth(path[0])
            .unwrap()
            .into_element()
            .unwrap();
        for ordinal in &path[1..] {
            element = element
                .children()
                .nth(*ordinal)
                .unwrap()
                .into_element()
                .unwrap();
        }
        let receipt = element.lint_tag().unwrap();
        assert!(receipt.in_table_context());
        assert_eq!(
            lint.deprecated_attr(&element, &NeverLookup).err(),
            Some(NativeLintRefusal::TableContext {
                span: receipt.span()
            })
        );
        for locale in LOCALES {
            let mut diagnostics = Vec::new();
            if foster {
                diagnostics.push(parser(
                    "error",
                    "Foster parenting moved this element before the nearest open table.",
                    span(source, "<div align='yes'>"),
                ));
            }
            diagnostics.push(warning(
                locale,
                span(source, head),
                tag,
                if foster { "align" } else { "width" },
                suggestion,
            ));
            assert_eq!(complete(&registered(source, locale)), expected(diagnostics));
        }
    }
    let source = "<template><table cellpadding='own'></table><div align='outside' /></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![
                warning(
                    locale,
                    span(source, "cellpadding='own'"),
                    "table",
                    "cellpadding",
                    "CSS `padding` on cells"
                ),
                warning(
                    locale,
                    span(source, "align='outside'"),
                    "div",
                    "align",
                    ALIGN
                )
            ])
        );
    }
}

#[test]
fn ignored_nested_form_keeps_parser_first_and_complete_nonempty_product_control() {
    let source = "<template><form><form align='yes' /></form></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let outer = owner.children().next().unwrap().into_element().unwrap();
    let element = outer.children().next().unwrap().into_element().unwrap();
    let receipt = element.lint_tag().unwrap();
    assert!(receipt.in_recovery_context());
    assert_eq!(
        lint.deprecated_attr(&element, &NeverLookup).err(),
        Some(NativeLintRefusal::RecoveryContext {
            span: receipt.span()
        })
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                parser(
                    "error",
                    "HTML tree construction ignored this start tag because an equivalent element is already open.",
                    span(source, "<form align='yes' />")
                ),
                warning(locale, span(source, "align='yes'"), "form", "align", ALIGN)
            ])
        );
    }
}

#[test]
fn supplied_missing_close_refuses_and_keeps_original_parser_and_warning() {
    let source = "<template><div align='yes'></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    assert_eq!(
        lint.deprecated_attr(&element, &NeverLookup).err(),
        Some(NativeLintRefusal::Hole)
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                parser(
                    "error",
                    "Element is missing end tag.",
                    span(source, "<div align='yes'>")
                ),
                warning(locale, span(source, "align='yes'"), "div", "align", ALIGN)
            ])
        );
    }
}
