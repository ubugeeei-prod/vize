use super::support::{
    LOCALES, NeverLookup, complete, expected, parser, registered, selected, span, warning,
};
use vize_l0::{Allocator, Span, cstr};
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn later_dynamic_object_and_unknown_headers_refuse_before_any_catalog_lookup() {
    for (head, unresolved) in [
        (":[key]", true),
        (".[key]", true),
        ("@[key]", true),
        ("v-bind:[key]", true),
        ("v-bind", true),
        ("v-bind.prop", true),
        ("v-unknown", false),
    ] {
        let opening = cstr!("<center title='first' {head}='value'>");
        let source = cstr!("<template>{opening}</center></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let refusal = if unresolved {
            NativeLintRefusal::UnresolvedBinding {
                span: span(&source, head),
            }
        } else {
            NativeLintRefusal::UnsupportedDirective {
                span: span(&source, head),
            }
        };
        assert_eq!(
            lint.deprecated_element(&element, &NeverLookup).err(),
            Some(refusal)
        );
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(vec![warning(locale, span(&source, &opening), "center")])
            );
        }
        assert!(owner.component().carrier().errors.is_empty());
    }
}

#[test]
fn empty_modifiers_keep_full_reportable_parser_output_and_strict_refusal() {
    for head in [
        ":title.",
        ":title..prop",
        ".title.",
        "v-bind:title..camel",
        "@click.",
    ] {
        for pre in ["", "before", "after"] {
            let header = match pre {
                "before" => cstr!("v-pre {head}='x'"),
                "after" => cstr!("{head}='x' v-pre"),
                _ => cstr!("{head}='x'"),
            };
            let opening = cstr!("<center {header}>");
            let source = cstr!("<template>{opening}</center></template>");
            let arena = Allocator::default();
            let owner = selected(&arena, &source);
            let lint = NativeSyntaxLint::new(&owner).unwrap();
            let element = owner.children().next().unwrap().into_element().unwrap();
            let range = span(&source, head);
            assert_eq!(
                lint.deprecated_element(&element, &NeverLookup).err(),
                Some(NativeLintRefusal::UnsupportedDirective { span: range })
            );
            let local_error = head.find("..").map_or(head.len(), |index| index + 1);
            let start = range.start + u32::try_from(local_error).unwrap();
            for locale in LOCALES {
                assert_eq!(
                    complete(&registered(&source, locale)),
                    expected(vec![
                        warning(locale, span(&source, &opening), "center"),
                        parser(
                            "error",
                            "Directive modifier is expected.",
                            Span::new(start, start + 1)
                        ),
                    ])
                );
            }
            assert!(owner.component().carrier().errors.is_empty());
        }
    }
}

#[test]
fn unrelated_static_duplicates_refuse_even_on_exempt_or_nonmatching_tags() {
    for tag in ["center", "Center", "div"] {
        let opening = cstr!("<{tag} id='one' ID='two'>");
        let source = cstr!("<template>{opening}</{tag}></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let repeated = span(&source, "ID");
        assert_eq!(
            lint.deprecated_element(&element, &NeverLookup).err(),
            Some(NativeLintRefusal::DuplicateAttribute { span: repeated })
        );
        for locale in LOCALES {
            let mut diagnostics = vec![parser(
                "warning",
                "Duplicate attribute `ID`. Keeping the repeated attribute so parsing can continue.",
                repeated,
            )];
            if tag == "center" {
                diagnostics.push(warning(locale, span(&source, &opening), tag));
            }
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(diagnostics)
            );
        }
    }
}

#[test]
fn inherited_literal_duplicates_preserve_the_original_parser_warning() {
    let opening = "<center :title='one' :TITLE='two'>";
    let source = cstr!("<template><div v-pre>{opening}</center></div></template>");
    let arena = Allocator::default();
    let owner = selected(&arena, &source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let element = parent.children().next().unwrap().into_element().unwrap();
    let repeated = span(&source, ":TITLE");
    assert_eq!(
        lint.deprecated_element(&element, &NeverLookup).err(),
        Some(NativeLintRefusal::DuplicateAttribute { span: repeated })
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(&source, locale)),
            expected(vec![
                warning(locale, span(&source, opening), "center"),
                parser(
                    "warning",
                    "Duplicate attribute `:TITLE`. Keeping the repeated attribute so parsing can continue.",
                    repeated
                ),
            ])
        );
    }
}
