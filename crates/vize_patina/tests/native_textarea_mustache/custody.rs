use super::support::{FILENAME, LOCALES, NeverLookup, complete, native, parity, selected};
use std::{borrow::Cow, cell::Cell};
use vize_l0::{Allocator, diag::MessageLookup};
use vize_patina::{
    Linter,
    native::{NativeSyntaxLint, child_facts::NativeChildFactError},
};

#[test]
fn equal_source_foreign_owner_is_refused_before_observation_or_lookup() {
    let source = "<template><textarea>{{x}}</textarea></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let foreign = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let other = foreign.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    assert_eq!(
        header.child_facts(other.children().next().unwrap()).err(),
        Some(NativeChildFactError::ForeignChild)
    );
}

#[test]
fn sibling_parent_with_same_tag_is_refused_even_when_its_child_ordinal_matches() {
    let source = "<template><textarea>{{one}}</textarea><textarea>{{two}}</textarea></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let sibling = owner.children().nth(1).unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    assert_eq!(
        header.child_facts(sibling.children().next().unwrap()).err(),
        Some(NativeChildFactError::WrongParent)
    );
}

#[test]
fn root_child_and_grandchild_cannot_mint_a_direct_child_marker() {
    let source = "<template><div><span>{{x}}</span></div>{{root}}</template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let nested = element.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    assert_eq!(
        header.child_facts(owner.children().nth(1).unwrap()).err(),
        Some(NativeChildFactError::WrongParent)
    );
    assert_eq!(
        header.child_facts(nested.children().next().unwrap()).err(),
        Some(NativeChildFactError::WrongParent)
    );
}

#[test]
fn owned_complete_errors_help_and_chains_outlive_arena_owner_and_catalog() {
    struct OwnedCatalog {
        calls: Cell<usize>,
    }
    impl MessageLookup for OwnedCatalog {
        fn lookup(&self, key: &str) -> Cow<'static, str> {
            self.calls.set(self.calls.get() + 1);
            Cow::Owned(
                match key {
                    "vue/no-textarea-mustache.message" => "complete owned message",
                    "vue/no-textarea-mustache.help" => "complete owned help\n\nall examples remain",
                    _ => panic!("unexpected catalog key"),
                }
                .to_owned(),
            )
        }
    }
    let (findings, before) = {
        let arena = Allocator::default();
        let source = "<template><textarea>{{}}</textarea></template>";
        let owner = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let header = lint.header_facts(&element).unwrap();
        let facts = header
            .child_facts(element.children().next().unwrap())
            .unwrap();
        let catalog = OwnedCatalog {
            calls: Cell::new(0),
        };
        let findings = facts.no_textarea_mustache(&catalog).unwrap();
        assert_eq!(catalog.calls.get(), 2);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            facts.verify(findings[0].diagnostic().witness_chain().unwrap()),
            Ok(())
        );
        let before = findings.iter().map(native).collect::<Vec<_>>();
        (findings, before)
    };
    assert_eq!(findings.iter().map(native).collect::<Vec<_>>(), before);
    let diagnostic = findings.into_iter().next().unwrap().into_diagnostic();
    assert_eq!(diagnostic.message.as_str(), "complete owned message");
    assert_eq!(
        diagnostic.parts[0].message.as_str(),
        "complete owned help\n\nall examples remain"
    );
    assert_eq!(diagnostic.witness_chain().unwrap().links().len(), 3);
}

#[test]
fn a_text_child_has_no_whole_body_marker_absence_authority() {
    let source = "<template><textarea>safe{{actual}}</textarea></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let header = lint.header_facts(&element).unwrap();
    let text = header
        .child_facts(element.children().next().unwrap())
        .unwrap();
    assert_eq!(text.markers().unwrap().len(), 0);
    assert_eq!(text.no_textarea_mustache(&NeverLookup).unwrap().len(), 0);
    let marker = header
        .child_facts(element.children().nth(1).unwrap())
        .unwrap();
    assert_eq!(marker.markers().unwrap().len(), 1);
    assert_eq!(marker.textarea_mustache().unwrap().len(), 1);
    for locale in LOCALES {
        assert_eq!(parity(source, locale)["error_count"], 1);
    }
}

#[test]
fn original_owner_can_move_after_receipt_borrows_end_and_default_route_is_unchanged() {
    let source = "<template><textarea>{{x}}</textarea></template>";
    let before = complete(&Linter::default().lint_sfc(source, FILENAME));
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    {
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let header = lint.header_facts(&element).unwrap();
        let facts = header
            .child_facts(element.children().next().unwrap())
            .unwrap();
        assert!(core::ptr::eq(facts.header().owner(), &owner));
        assert!(core::ptr::eq(
            facts.original().surface(),
            element.children().next().unwrap().surface()
        ));
    }
    let moved = owner;
    assert_eq!(
        moved.component().carrier().tree.source,
        "<textarea>{{x}}</textarea>"
    );
    for locale in LOCALES {
        assert_eq!(parity(source, locale)["error_count"], 1);
    }
    assert_eq!(
        complete(&Linter::default().lint_sfc(source, FILENAME)),
        before
    );
}
