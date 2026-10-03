use super::support::{
    LOCALES, NeverLookup, complete, expected, options, parser, registered, selected, span, warning,
};
use vize_l0::{Allocator, cstr};
use vize_l1::{
    check_fidelity,
    container::{Vue, vue::DescriptorIssueCode},
};
use vize_patina::native::{NativeLintRefusal, NativeSyntaxLint};

#[test]
fn authored_table_context_refuses_and_retains_complete_original_foster_output() {
    let opening = "<center title='x'>";
    let source = cstr!("<template><table>{opening}</center></table></template>");
    let arena = Allocator::default();
    let owner = selected(&arena, &source);
    let before = cstr!("{:?}", owner.component().carrier());
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let table = owner.children().next().unwrap().into_element().unwrap();
    let element = table.children().next().unwrap().into_element().unwrap();
    let receipt = element.lint_tag().unwrap();
    assert!(receipt.in_table_context());
    assert_eq!(
        lint.deprecated_element(&element, &NeverLookup).err(),
        Some(NativeLintRefusal::TableContext {
            span: receipt.span()
        })
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(&source, locale)),
            expected(vec![
                parser(
                    "error",
                    "Foster parenting moved this element before the nearest open table.",
                    span(&source, opening)
                ),
                warning(locale, span(&source, opening), "center"),
            ])
        );
    }
    assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
}

#[test]
fn valid_cells_keep_conservative_table_refusal_and_the_original_product_warning() {
    let source = "<template><table><tbody><tr><td><center /></td></tr></tbody></table></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let table = owner.children().next().unwrap().into_element().unwrap();
    let body = table.children().next().unwrap().into_element().unwrap();
    let row = body.children().next().unwrap().into_element().unwrap();
    let cell = row.children().next().unwrap().into_element().unwrap();
    let element = cell.children().next().unwrap().into_element().unwrap();
    assert_eq!(
        lint.deprecated_element(&element, &NeverLookup).err(),
        Some(NativeLintRefusal::TableContext {
            span: element.lint_tag().unwrap().span()
        })
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![warning(locale, span(source, "<center />"), "center")])
        );
    }
}

#[test]
fn actual_pre_recovery_context_refuses_without_granting_whole_file_completion() {
    let source = "<template><p v-pre><div><center /></div></p></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let paragraph = owner.children().next().unwrap().into_element().unwrap();
    let parent = paragraph.children().next().unwrap().into_element().unwrap();
    let element = parent.children().next().unwrap().into_element().unwrap();
    let receipt = element.lint_tag().unwrap();
    assert!(receipt.in_recovery_context());
    assert_eq!(
        lint.deprecated_element(&element, &NeverLookup).err(),
        Some(NativeLintRefusal::RecoveryContext {
            span: receipt.span()
        })
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                warning(locale, span(source, "<center />"), "center"),
                parser("error", "Invalid end tag.", span(source, "</p>")),
            ])
        );
    }
}

#[test]
fn missing_supplied_close_refuses_without_deleting_original_parser_or_product_diagnostics() {
    let source = "<template><center></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    assert_eq!(
        lint.deprecated_element(&element, &NeverLookup).err(),
        Some(NativeLintRefusal::Hole)
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                parser(
                    "error",
                    "Element is missing end tag.",
                    span(source, "<center>")
                ),
                warning(locale, span(source, "<center>"), "center"),
            ])
        );
    }
}

#[test]
fn original_lexical_recovery_is_retained_and_cannot_be_admitted_by_the_consumer() {
    let source = "<template><center title='unterminated /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let before = cstr!("{:?}", owner.component().carrier());
    let error = owner.component().carrier().errors.first().unwrap();
    let offset = owner.component().block().start() + error.offset;
    assert_eq!(
        NativeSyntaxLint::new(&owner).err(),
        Some(NativeLintRefusal::Recovered { offset })
    );
    assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
    assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
}

#[test]
fn uncertain_original_descriptor_keeps_its_refusal_without_a_synthetic_owner() {
    let source = "<template><xmp>{{ `prefix ${value}` }}</xmp></template>";
    let arena = Allocator::default();
    let observed = Vue.observe_descriptor(&arena, source, options());
    let refusal = observed
        .admitted()
        .expect_err("actual descriptor boundary refusal");
    assert_eq!(refusal.errors(), &*observed.container().errors);
    assert_eq!(
        refusal
            .issues()
            .iter()
            .map(|issue| issue.code)
            .collect::<Vec<_>>(),
        vec![DescriptorIssueCode::UnsupportedBoundary]
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![warning(locale, span(source, "<xmp>"), "xmp")])
        );
    }
}
