use super::support::selected;
use vize_l0::{Allocator, cstr};
use vize_l1::{check_fidelity, render};
use vize_patina::{
    Linter,
    native::{NativeLintRefusal, NativeSyntaxLint, header_facts::NativeHeaderFactError},
};

#[test]
fn equal_foreign_owners_cannot_establish_fact_custody() {
    let source = "<template><meta role='presentation' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let foreign = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let element = foreign.children().next().unwrap().into_element().unwrap();
    assert_eq!(
        lint.header_facts(&element).err(),
        Some(NativeHeaderFactError::Header(
            NativeLintRefusal::ForeignElement
        ))
    );
    assert!(core::ptr::eq(element.component(), foreign.component()));
    assert!(core::ptr::eq(lint.owner(), &owner));
}

#[test]
fn facts_keep_original_owner_source_ranges_render_and_default_linter_unchanged() {
    let source = "<template><meta aria-hidden='字' :role='opaque' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let before = cstr!("{:?}", owner.component().carrier());
    let default_before = cstr!("{:?}", Linter::default().lint_sfc(source, "facts.vue"));
    let original_source = owner.component().carrier().tree.source;
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let facts = lint.header_facts(&element).unwrap();
    assert!(core::ptr::eq(facts.owner(), &owner));
    assert!(core::ptr::eq(facts.original(), element.surface()));
    for (key, _) in facts.unsupported_aria().unwrap().iter() {
        assert_eq!(
            facts.verify(&facts.unsupported_aria_chain(*key).unwrap().unwrap()),
            Ok(())
        );
    }
    assert!(core::ptr::eq(
        original_source,
        owner.component().carrier().tree.source
    ));
    assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
    assert_eq!(
        cstr!("{:?}", Linter::default().lint_sfc(source, "facts.vue")),
        default_before
    );
    let mut output = String::new();
    render(&owner.component().carrier().tree, &mut |piece| {
        output.push_str(piece)
    });
    assert_eq!(output, owner.component().block().source());
    assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
}

#[test]
fn actual_owner_recovery_is_retained_without_fabricating_facts() {
    let source = "<template><meta role='unterminated /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let before = cstr!("{:?}", owner.component().carrier());
    let error = owner.component().carrier().errors.first().unwrap();
    assert_eq!(
        NativeSyntaxLint::new(&owner).err(),
        Some(NativeLintRefusal::Recovered {
            offset: owner.component().block().start() + error.offset
        })
    );
    assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
    assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
}
