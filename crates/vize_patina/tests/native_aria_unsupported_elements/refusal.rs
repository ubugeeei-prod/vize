use super::support::{LOCALES, complete, error, expected, parser, registered, selected, span};
use vize_l0::{Allocator, Span, cstr};
use vize_l1::markup::NativeLintTagRefusal;
use vize_patina::native::{
    NativeLintRefusal, NativeSyntaxLint, header_facts::NativeHeaderFactError,
};

#[test]
fn later_dynamic_object_and_custom_heads_refuse_without_omitting_original_errors() {
    for (head, unresolved) in [
        ("v-bind", true),
        (":[key]", true),
        ("v-bind:[key]", true),
        ("@[key]", true),
        ("v-unknown", false),
    ] {
        let source = cstr!("<template><meta role {head}='x' /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let range = span(&source, head);
        let refusal = if unresolved {
            NativeLintRefusal::UnresolvedBinding { span: range }
        } else {
            NativeLintRefusal::UnsupportedDirective { span: range }
        };
        assert_eq!(
            lint.header_facts(&element).err(),
            Some(NativeHeaderFactError::Header(refusal))
        );
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(vec![error(locale, span(&source, "role"), "meta", "role")])
            );
        }
    }
}

#[test]
fn malformed_modifier_refusal_keeps_complete_registered_error_and_parser_error() {
    for head in [
        ":title.",
        ":title..prop",
        ".title.",
        "@click.",
        "v-bind:title..camel",
    ] {
        for pre in ["", "v-pre "] {
            let source = cstr!("<template><meta {pre}role {head}='x' /></template>");
            let arena = Allocator::default();
            let owner = selected(&arena, &source);
            let element = owner.children().next().unwrap().into_element().unwrap();
            let lint = NativeSyntaxLint::new(&owner).unwrap();
            let range = span(&source, head);
            assert_eq!(
                lint.header_facts(&element).err(),
                Some(NativeHeaderFactError::Header(
                    NativeLintRefusal::UnsupportedDirective { span: range }
                ))
            );
            let start = range.start
                + u32::try_from(head.find("..").map_or(head.len(), |index| index + 1)).unwrap();
            for locale in LOCALES {
                assert_eq!(
                    complete(&registered(&source, locale)),
                    expected(vec![
                        error(locale, span(&source, "role"), "meta", "role"),
                        parser(
                            "error",
                            "Directive modifier is expected.",
                            Span::new(start, start + 1)
                        ),
                    ])
                );
            }
        }
    }
}

#[test]
fn repeated_original_attribute_notice_requires_whole_result_refusal() {
    let source = "<template><meta role='one' ROLE='two' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let repeated = span(source, "ROLE");
    assert_eq!(
        lint.header_facts(&element).err(),
        Some(NativeHeaderFactError::Header(
            NativeLintRefusal::DuplicateAttribute { span: repeated }
        ))
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                error(locale, span(source, "role='one'"), "meta", "role"),
                parser(
                    "warning",
                    "Duplicate attribute `ROLE`. Keeping the repeated attribute so parsing can continue.",
                    repeated
                ),
            ])
        );
    }
}

#[test]
fn original_modified_pre_classification_stays_ambiguous() {
    for head in ["v-pre.foo", "v-pre:argument", "v-pre:[argument]"] {
        let source = cstr!("<template><meta {head} role /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        assert_eq!(
            lint.header_facts(&element).err(),
            Some(NativeHeaderFactError::Header(NativeLintRefusal::LintTag {
                reason: NativeLintTagRefusal::AmbiguousVerbatim
            }))
        );
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(vec![error(locale, span(&source, "role"), "meta", "role")])
            );
        }
    }
}
