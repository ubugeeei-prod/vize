use super::support::{
    FILENAME, HeaderRule, LOCALES, NeverLookup, complete, native, parity, selected,
};
use std::{borrow::Cow, cell::Cell};
use vize_l0::{Allocator, cstr, diag::MessageLookup};
use vize_l1::check_fidelity;
use vize_patina::{
    Linter,
    native::{NativeLintRefusal, NativeSyntaxLint},
};

#[test]
fn equal_foreign_owners_refuse_before_any_lookup_and_original_custody_survives() {
    let source = "<template><marquee autofocus accesskey /></template>";
    let arena = Allocator::default();
    let first = selected(&arena, source);
    let second = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&first).unwrap();
    let element = second.children().next().unwrap().into_element().unwrap();
    for rule in HeaderRule::ALL {
        assert_eq!(
            rule.check(&lint, &element, &NeverLookup).err(),
            Some(NativeLintRefusal::ForeignElement)
        );
    }
    assert!(core::ptr::eq(lint.owner(), &first));
    assert!(core::ptr::eq(element.component(), second.component()));
    assert_eq!(
        first.component().carrier().tree.source,
        second.component().carrier().tree.source
    );
    assert_eq!(check_fidelity(&first.component().carrier().tree), Ok(()));
    assert_eq!(check_fidelity(&second.component().carrier().tree), Ok(()));
}

#[test]
fn recovery_is_read_from_original_owner_without_clearing_its_observations() {
    let source = "<template><marquee autofocus accesskey title='unterminated /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let before = cstr!("{:?}", owner.component().carrier());
    let errors = &owner.component().carrier().errors;
    assert!(!errors.is_empty());
    let offset = owner.component().block().start() + errors[0].offset;
    assert_eq!(
        NativeSyntaxLint::new(&owner).err(),
        Some(NativeLintRefusal::Recovered { offset })
    );
    assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
    assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
}

#[test]
fn owned_findings_outlive_original_owners_arenas_and_message_providers() {
    struct OwnedCatalog {
        calls: Cell<usize>,
    }
    impl MessageLookup for OwnedCatalog {
        fn lookup(&self, key: &str) -> Cow<'static, str> {
            self.calls.set(self.calls.get() + 1);
            Cow::Owned(
                if key.ends_with(".message") {
                    "owned {tag} message"
                } else {
                    "owned full help"
                }
                .to_owned(),
            )
        }
    }
    for rule in HeaderRule::ALL {
        let (findings, before) = {
            let arena = Allocator::default();
            let source = "<template><marquee autofocus accesskey /></template>";
            let owner = selected(&arena, source);
            let lint = NativeSyntaxLint::new(&owner).unwrap();
            let element = owner.children().next().unwrap().into_element().unwrap();
            let messages = OwnedCatalog {
                calls: Cell::new(0),
            };
            let findings = rule.check(&lint, &element, &messages).unwrap();
            assert_eq!(messages.calls.get(), 2);
            assert_eq!(findings.len(), 1);
            let before = findings.iter().map(native).collect::<Vec<_>>();
            (findings, before)
        };
        assert_eq!(findings.iter().map(native).collect::<Vec<_>>(), before);
        assert_eq!(
            findings.into_iter().next().unwrap().into_diagnostic().parts[0]
                .message
                .as_str(),
            "owned full help"
        );
    }
}

#[test]
fn component_exemption_keeps_the_catalog_unused_after_a_complete_valid_header() {
    let source = "<template><Foo autofocus accesskey :title.prop='value' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    for rule in HeaderRule::ALL {
        assert!(
            rule.check(&lint, &element, &NeverLookup)
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn opt_in_calls_leave_complete_default_sfc_results_unchanged() {
    let source = "<!--🦀--><template><marquee autofocus accesskey='x'><blink autofocus accesskey /></marquee></template>";
    for locale in LOCALES {
        let linter = Linter::default().with_locale(locale);
        let before = complete(&linter.lint_sfc(source, FILENAME));
        for rule in HeaderRule::ALL {
            parity(source, locale, rule);
        }
        assert_eq!(complete(&linter.lint_sfc(source, FILENAME)), before);
    }
}
