use super::support::{LOCALES, NeverLookup, complete, native, parity, selected};
use std::{borrow::Cow, cell::Cell};
use vize_l0::{Allocator, cstr, diag::MessageLookup};
use vize_patina::{Linter, native::NativeSyntaxLint};

#[test]
fn completed_proven_errors_and_full_help_outlive_the_original_owner_arena_and_catalog() {
    struct OwnedCatalog {
        calls: Cell<usize>,
    }
    impl MessageLookup for OwnedCatalog {
        fn lookup(&self, key: &str) -> Cow<'static, str> {
            self.calls.set(self.calls.get() + 1);
            Cow::Owned(
                if key.ends_with(".message") {
                    "{tag}:{attr}:{tag}:{attr}:{other}"
                } else {
                    "owned complete help"
                }
                .to_owned(),
            )
        }
    }
    let (findings, before) = {
        let arena = Allocator::default();
        let source = "<template><meta aria-{tag} /></template>";
        let owner = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let facts = lint.header_facts(&element).unwrap();
        let messages = OwnedCatalog {
            calls: Cell::new(0),
        };
        let findings = facts.aria_unsupported_elements(&messages).unwrap();
        assert_eq!(messages.calls.get(), 2);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].diagnostic().message.as_str(),
            "meta:aria-{tag}:meta:aria-{tag}:{other}"
        );
        assert_eq!(
            facts.verify(findings[0].diagnostic().witness_chain().unwrap()),
            Ok(())
        );
        let before = findings.iter().map(native).collect::<Vec<_>>();
        (findings, before)
    };
    assert_eq!(findings.iter().map(native).collect::<Vec<_>>(), before);
    let diagnostic = findings.into_iter().next().unwrap().into_diagnostic();
    assert_eq!(diagnostic.parts[0].message.as_str(), "owned complete help");
    assert_eq!(diagnostic.witness_chain().unwrap().links().len(), 3);
}

#[test]
fn original_exempt_headers_cannot_touch_a_catalog() {
    for source in [
        "<template><Meta role /></template>",
        "<template><meta title='ok' /></template>",
        "<template><div aria-hidden /></template>",
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let facts = lint.header_facts(&element).unwrap();
        assert!(
            facts
                .aria_unsupported_elements(&NeverLookup)
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn default_registered_routes_remain_byte_exact_around_opt_in_execution() {
    let source = "<template><meta role aria-hidden :role='x' /></template>";
    let before = complete(&Linter::default().lint_sfc(source, "native-aria.vue"));
    for locale in LOCALES {
        assert_eq!(parity(source, locale)["error_count"], 3);
    }
    assert_eq!(
        complete(&Linter::default().lint_sfc(source, "native-aria.vue")),
        before
    );
    assert_eq!(
        cstr!("{:?}", before),
        cstr!(
            "{:?}",
            complete(&Linter::default().lint_sfc(source, "native-aria.vue"))
        )
    );
}
