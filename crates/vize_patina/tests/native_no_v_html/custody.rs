use super::support::{FILENAME, LOCALES, NeverLookup, complete, native, parity, selected};
use std::{borrow::Cow, cell::Cell};
use vize_l0::{Allocator, cstr, diag::MessageLookup};
use vize_l1::check_fidelity;
use vize_patina::{
    Linter,
    native::{NativeLintRefusal, NativeSyntaxLint},
};

#[test]
fn equal_foreign_owners_refuse_before_lookup_and_preserve_original_custody() {
    let source = "<template><Foo v-html='yes' /></template>";
    let arena = Allocator::default();
    let first = selected(&arena, source);
    let second = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&first).unwrap();
    let element = second.children().next().unwrap().into_element().unwrap();
    assert_eq!(
        lint.no_v_html(&element, &NeverLookup).err(),
        Some(vize_patina::native::NativeVHtmlLintError::Header(
            NativeLintRefusal::ForeignElement,
        ))
    );
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
fn original_recovery_observations_refuse_without_being_cleared() {
    let source = "<template><Foo v-html='yes' title='unterminated /></template>";
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
fn owned_findings_outlive_original_owners_arenas_and_catalogs() {
    struct OwnedCatalog {
        calls: Cell<usize>,
    }
    impl MessageLookup for OwnedCatalog {
        fn lookup(&self, key: &str) -> Cow<'static, str> {
            self.calls.set(self.calls.get() + 1);
            Cow::Owned(
                match key {
                    "vue/no-v-html.message" => "owned HTML message",
                    "vue/no-v-html.help" => "owned full help",
                    _ => panic!("unexpected catalog key"),
                }
                .to_owned(),
            )
        }
    }
    let (findings, before) = {
        let arena = Allocator::default();
        let source = "<template><Foo v-html='yes' /></template>";
        let owner = selected(&arena, source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let messages = OwnedCatalog {
            calls: Cell::new(0),
        };
        let findings = lint.no_v_html(&element, &messages).unwrap();
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

#[test]
fn clean_static_and_bound_headers_do_not_lookup_any_message() {
    let source = "<template><Foo innerHTML='x' :style='broken(' .title='opaque' v-text='broken(' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    assert!(lint.no_v_html(&element, &NeverLookup).unwrap().is_empty());
}

#[test]
fn opt_in_calls_preserve_complete_default_registered_results() {
    let source =
        "<!--🦀--><template><Foo v-html='one' :style='broken(' /><div v-html='two' /></template>";
    for locale in LOCALES {
        let linter = Linter::default().with_locale(locale);
        let before = complete(&linter.lint_sfc(source, FILENAME));
        parity(source, locale);
        assert_eq!(complete(&linter.lint_sfc(source, FILENAME)), before);
    }
}
