use super::support::{
    LOCALES, NeverLookup, complete, expected, parser, registered, selected, span, warning,
};
use vize_l0::{Allocator, Span, cstr};
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint, NativeVHtmlLintError};

#[test]
fn late_dynamic_object_custom_and_empty_modifier_heads_refuse_every_partial_match() {
    for (head, unresolved) in [
        (":[key]", true),
        (".[key]", true),
        ("@[key]", true),
        ("v-bind:[key]", true),
        ("v-bind", true),
        ("v-bind.prop", true),
        ("v-unknown", false),
        ("v-htmlx-lookalike", false),
        ("v-HTML", false),
        (":title.", false),
    ] {
        let source = cstr!("<template><Foo v-html='first' {head}='broken(' /></template>");
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
            lint.no_v_html(&element, &NeverLookup).err(),
            Some(NativeVHtmlLintError::Header(refusal))
        );
        for locale in LOCALES {
            let mut diagnostics = vec![warning(locale, span(&source, "v-html='first'"))];
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

#[test]
fn full_html_dynamic_arguments_keep_original_whole_directive_warning_when_refused() {
    let source = "<template><div v-html='first' v-html:[key]='last' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    assert_eq!(
        lint.no_v_html(&element, &NeverLookup).err(),
        Some(NativeVHtmlLintError::Header(
            NativeLintRefusal::UnresolvedBinding {
                span: span(source, "v-html:[key]")
            }
        ))
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                warning(locale, span(source, "v-html='first'")),
                warning(locale, span(source, "v-html:[key]='last'")),
            ])
        );
    }
}

#[test]
fn literal_dynamic_sinks_preserve_original_argument_diagnostics_as_refused_controls() {
    for (head, argument) in [
        (":['innerHTML']", "'innerHTML'"),
        ("v-bind:[`outerHTML`].prop", "`outerHTML`"),
        (".[\"outerHTML\"]", "\"outerHTML\""),
    ] {
        let source = cstr!("<template><div v-html='first' {head}='x' /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        assert_eq!(
            lint.no_v_html(&element, &NeverLookup).err(),
            Some(NativeVHtmlLintError::Header(
                NativeLintRefusal::UnresolvedBinding {
                    span: span(&source, head)
                }
            ))
        );
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(vec![
                    warning(locale, span(&source, "v-html='first'")),
                    warning(locale, span(&source, argument)),
                ])
            );
        }
    }
}

#[test]
fn object_spread_and_computed_literal_keys_keep_complete_original_positive_vectors() {
    for (value, keys) in [
        (
            "{ innerHTML: html, ...{ outerHTML: html } }",
            &["innerHTML", "outerHTML"][..],
        ),
        (
            "{ 'innerHTML': html, ['outerHTML']: html }",
            &["'innerHTML'", "'outerHTML'"][..],
        ),
        ("({ innerHTML })", &["innerHTML"][..]),
        ("{ ...{ [`outerHTML`]: html } }", &["`outerHTML`"][..]),
    ] {
        let source = cstr!("<template><div v-html='first' v-bind=\"{value}\" /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        assert_eq!(
            lint.no_v_html(&element, &NeverLookup).err(),
            Some(NativeVHtmlLintError::Header(
                NativeLintRefusal::UnresolvedBinding {
                    span: span(&source, "v-bind")
                }
            ))
        );
        for locale in LOCALES {
            let mut diagnostics = vec![warning(locale, span(&source, "v-html='first'"))];
            diagnostics.extend(keys.iter().map(|key| warning(locale, span(&source, key))));
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(diagnostics)
            );
        }
    }
}

#[test]
fn unknown_dynamic_and_object_expressions_are_refused_even_when_original_is_clean() {
    for head in [
        ":[innerHTML]='x'",
        "v-bind='html'",
        "v-bind='{ ...html }'",
        "v-bind='{ innerhtml: html }'",
        "v-bind='{ [innerHTML]: html }'",
        "v-bind='{ class: html }'",
    ] {
        let source = cstr!("<template><div {head} /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let name = head.split('=').next().unwrap();
        assert_eq!(
            lint.no_v_html(&element, &NeverLookup).err(),
            Some(NativeVHtmlLintError::Header(
                NativeLintRefusal::UnresolvedBinding {
                    span: span(&source, name)
                }
            ))
        );
        for locale in LOCALES {
            assert_eq!(complete(&registered(&source, locale)), expected(vec![]));
        }
    }
}

#[test]
fn duplicate_static_attributes_preserve_parser_warning_and_product_order() {
    let source = "<template><Foo v-html='first' id='one' ID='two' :outerHTML='last' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let repeated = span(source, "ID");
    assert_eq!(
        lint.no_v_html(&element, &NeverLookup).err(),
        Some(NativeVHtmlLintError::Header(
            NativeLintRefusal::DuplicateAttribute { span: repeated }
        ))
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                warning(locale, span(source, "v-html='first'")),
                parser(
                    "warning",
                    "Duplicate attribute `ID`. Keeping the repeated attribute so parsing can continue.",
                    repeated
                ),
                warning(locale, span(source, "outerHTML")),
            ])
        );
    }
}

#[test]
fn malformed_sink_modifiers_keep_original_warning_and_parser_error_order() {
    for head in [
        "v-html.",
        "v-html..foo",
        ":innerHTML.",
        ".outerHTML..prop",
        "v-bind:innerHTML.",
    ] {
        let source = cstr!("<template><div {head}='x' /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let range = span(&source, head);
        assert_eq!(
            lint.no_v_html(&element, &NeverLookup).err(),
            Some(NativeVHtmlLintError::Header(
                NativeLintRefusal::UnsupportedDirective { span: range }
            ))
        );
        let offset =
            range.start + u32::try_from(head.find("..").map_or(head.len(), |i| i + 1)).unwrap();
        for locale in LOCALES {
            let sink = if head.starts_with("v-html") {
                span(&source, &cstr!("{head}='x'"))
            } else if head.contains("innerHTML") {
                span(&source, "innerHTML")
            } else {
                span(&source, "outerHTML")
            };
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(vec![
                    warning(locale, sink),
                    parser(
                        "error",
                        "Directive modifier is expected.",
                        Span::new(offset, offset + 1)
                    ),
                ])
            );
        }
    }
}

#[test]
fn missing_original_close_refuses_before_lookup_with_complete_registered_vector() {
    let source = "<template><div v-html='html'></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    assert_eq!(
        lint.no_v_html(&element, &NeverLookup).err(),
        Some(NativeVHtmlLintError::Header(NativeLintRefusal::Hole))
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                parser(
                    "error",
                    "Element is missing end tag.",
                    span(source, "<div v-html='html'>")
                ),
                warning(locale, span(source, "v-html='html'")),
            ])
        );
    }
}
