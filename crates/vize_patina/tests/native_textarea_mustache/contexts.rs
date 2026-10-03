use super::support::{
    LOCALES, NeverLookup, collect, complete, error, expected, parser, registered, selected, span,
};
use vize_l0::{Allocator, cstr};
use vize_patina::native::{
    NativeLintRefusal, NativeSyntaxLint, header_facts::NativeHeaderFactError,
};

fn table_refused(source: &str, depth: usize) {
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let before = cstr!("{:?}", owner.component().carrier());
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
    assert_eq!(receipt.in_table_context(), true);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    assert_eq!(
        lint.header_facts(&element).err(),
        Some(NativeHeaderFactError::Header(
            NativeLintRefusal::TableContext {
                span: receipt.span()
            }
        ))
    );
    let mut findings = Vec::new();
    assert_eq!(
        collect(&lint, owner.children(), &NeverLookup, &mut findings),
        Err(NativeHeaderFactError::Header(
            NativeLintRefusal::TableContext {
                span: first_context.unwrap(),
            }
        ))
    );
    assert_eq!(findings.len(), 0);
    assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
}

#[test]
fn original_foster_parenting_control_keeps_both_complete_errors() {
    let source = "<template><table><textarea>{{x}}</textarea></table></template>";
    table_refused(source, 2);
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                parser(
                    "error",
                    "Foster parenting moved this element before the nearest open table.",
                    span(source, "<textarea>")
                ),
                error(locale, span(source, "{{x}}")),
            ])
        );
    }
}

#[test]
fn valid_table_cell_ancestor_remains_outside_conservative_header_admission() {
    let source = "<template><table><tbody><tr><td><textarea>{{x}}</textarea></td></tr></tbody></table></template>";
    table_refused(source, 5);
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![error(locale, span(source, "{{x}}"))])
        );
    }
}

#[test]
fn genuine_pre_recovery_context_is_refused_without_fabricating_children() {
    let source = "<template><a v-pre><a /><textarea>{{x}}</textarea></a></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let element = parent.children().nth(1).unwrap().into_element().unwrap();
    let receipt = element.lint_tag().unwrap();
    assert_eq!(receipt.in_recovery_context(), true);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    assert_eq!(
        lint.header_facts(&element).err(),
        Some(NativeHeaderFactError::Header(
            NativeLintRefusal::RecoveryContext {
                span: receipt.span()
            }
        ))
    );
    for locale in LOCALES {
        assert_eq!(complete(&registered(source, locale)), expected(vec![]));
    }
}

#[test]
fn unchanged_header_registry_still_verifies_aria_and_rejects_child_only_groups() {
    use vize_l0::{
        diag::{WitnessChain, WitnessLink, verify::WitnessError},
        fact::FactGroup,
    };
    use vize_patina::native::child_facts::NativeDirectInterpolations;
    let source = "<template><meta role /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    let chain = header.unsupported_aria_chain(0).unwrap().unwrap();
    assert_eq!(header.verify(&chain), Ok(()));
    let group = NativeDirectInterpolations::ID;
    let child_chain = WitnessChain::new(WitnessLink::of::<NativeDirectInterpolations>(
        &0,
        span(source, "role"),
    ));
    assert_eq!(
        header.verify(&child_chain),
        Err(WitnessError::UnknownGroup { link: 0, group })
    );
}
