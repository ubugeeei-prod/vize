use super::support::{
    FILENAME, LOCALES, NeverLookup, complete, expected, native, parity, selected, span, warning,
};
use std::{borrow::Cow, cell::Cell};
use vize_l0::{Allocator, cstr, diag::MessageLookup};
use vize_l1::check_fidelity;
use vize_patina::{
    Linter,
    native::{NativeLintRefusal, NativeSyntaxLint},
};

#[test]
fn equal_foreign_owners_refuse_before_lookup_and_preserve_original_source() {
    let source = "<template><div align='yes' /></template>";
    let arena = Allocator::default();
    let first = selected(&arena, source);
    let second = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&first).unwrap();
    let element = second.children().next().unwrap().into_element().unwrap();
    assert_eq!(
        lint.deprecated_attr(&element, &NeverLookup).err(),
        Some(NativeLintRefusal::ForeignElement)
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
fn lexical_recovery_refuses_without_clearing_original_observations() {
    let source = "<template><div align='yes' title='unterminated /></template>";
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
fn owned_output_preserves_attr_then_tag_repeated_replacements_after_arena_and_catalog_drop() {
    struct OwnedCatalog {
        calls: Cell<usize>,
    }
    impl MessageLookup for OwnedCatalog {
        fn lookup(&self, key: &str) -> Cow<'static, str> {
            self.calls.set(self.calls.get() + 1);
            Cow::Owned(
                match key {
                    "html/deprecated-attr.message" => "{attr}:{tag}:{attr}:{tag}:{other}",
                    "html/deprecated-attr.help" => "{suggestion}::{suggestion}::{attr}",
                    _ => panic!("unexpected catalog key"),
                }
                .to_owned(),
            )
        }
    }
    let (findings, before) = {
        let arena = Allocator::default();
        let source = "<template><a{attr} align='x'></a{attr}></template>";
        let owner = selected(&arena, source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let messages = OwnedCatalog {
            calls: Cell::new(0),
        };
        let findings = lint.deprecated_attr(&element, &messages).unwrap();
        assert_eq!(messages.calls.get(), 2);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].diagnostic().message.as_str(),
            "align:a{attr}:align:a{attr}:{other}"
        );
        assert_eq!(
            findings[0].diagnostic().parts[0].message.as_str(),
            "CSS `text-align` or `margin: auto`::CSS `text-align` or `margin: auto`::{attr}"
        );
        let before = findings.iter().map(native).collect::<Vec<_>>();
        (findings, before)
    };
    assert_eq!(findings.iter().map(native).collect::<Vec<_>>(), before);
    assert_eq!(
        findings
            .into_iter()
            .next()
            .unwrap()
            .into_diagnostic()
            .message
            .as_str(),
        "align:a{attr}:align:a{attr}:{other}"
    );
}

#[test]
fn authored_tag_placeholders_remain_literal_in_all_registered_locale_messages() {
    let source = "<template><a{attr} align='x'></a{attr}></template>";
    for locale in LOCALES {
        assert_eq!(
            parity(source, locale),
            expected(vec![warning(
                locale,
                span(source, "align='x'"),
                "a{attr}",
                "align",
                "CSS `text-align` or `margin: auto`",
            )])
        );
    }
}

#[test]
fn complete_clean_or_exempt_headers_never_touch_a_catalog() {
    for source in [
        "<template><Foo align bgcolor /></template>",
        "<template><div Align='x' :align='broken(' .title='opaque' /></template>",
        "<template><table border='1'></table></template>",
        "<template><div background='x' /></template>",
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        assert!(
            lint.deprecated_attr(&element, &NeverLookup)
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn opt_in_execution_preserves_all_default_registered_product_fields() {
    let source =
        "<!--🦀--><template><Foo align='one' /><div align='two' :align='broken(' /></template>";
    for locale in LOCALES {
        let linter = Linter::default().with_locale(locale);
        let before = complete(&linter.lint_sfc(source, FILENAME));
        parity(source, locale);
        assert_eq!(complete(&linter.lint_sfc(source, FILENAME)), before);
    }
}
