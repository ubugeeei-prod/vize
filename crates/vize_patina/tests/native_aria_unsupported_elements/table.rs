use super::support::{
    LOCALES, NeverLookup, collect, complete, error, expected, parser, registered, selected, span,
};
use vize_l0::{Allocator, cstr};
use vize_patina::native::{
    NativeLintRefusal, NativeSyntaxLint, header_facts::NativeHeaderFactError,
};

const FOSTER: &str = "Foster parenting moved this element before the nearest open table.";

fn refused(source: &str, depth: usize) {
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let before = cstr!("{:?}", owner.component().carrier());
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let mut element = owner.children().next().unwrap().into_element().unwrap();
    let mut first_context = None;
    for _ in 1..depth {
        element = element.children().next().unwrap().into_element().unwrap();
        let original = element.lint_tag().unwrap();
        if original.in_table_context() {
            first_context.get_or_insert(original.span());
        }
    }
    let receipt = element.lint_tag().unwrap();
    assert!(receipt.in_table_context());
    let refusal = NativeHeaderFactError::Header(NativeLintRefusal::TableContext {
        span: receipt.span(),
    });
    assert_eq!(lint.header_facts(&element).err(), Some(refusal));
    let mut results = Vec::new();
    assert_eq!(
        collect(&lint, owner.children(), &NeverLookup, &mut results),
        Err(NativeHeaderFactError::Header(
            NativeLintRefusal::TableContext {
                span: first_context.unwrap(),
            }
        ))
    );
    assert!(results.is_empty());
    assert!(owner.component().carrier().errors.is_empty());
    assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
}

#[test]
fn preserved_original_table_html_role_keeps_both_registered_errors() {
    let source = "<template><table><html role></html></table></template>";
    refused(source, 2);
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                parser("error", FOSTER, span(source, "<html role>")),
                error(locale, span(source, "role"), "html", "role"),
            ])
        );
    }
}

#[test]
fn original_annotation_xml_namespace_ambiguity_keeps_the_complete_foster_error() {
    let source = "<template><math><annotation-xml encoding='text/html'><table><html role></html></table></annotation-xml></math></template>";
    refused(source, 4);
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                parser("error", FOSTER, span(source, "<html role>")),
                error(locale, span(source, "role"), "html", "role"),
            ])
        );
    }
}

#[test]
fn valid_table_cells_remain_explicitly_outside_this_conservative_admission() {
    let source =
        "<template><table><tbody><tr><td><html role></html></td></tr></tbody></table></template>";
    refused(source, 5);
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![error(locale, span(source, "role"), "html", "role")])
        );
    }
}
