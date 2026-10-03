use super::support::{
    FILENAME, LOCALES, NeverLookup, complete, expected, messages, native, parity, parser,
    registered, selected, span, warning,
};
use std::{borrow::Cow, cell::Cell};
use vize_l0::{Allocator, cstr, diag::MessageLookup};
use vize_l1::{
    check_fidelity,
    markup::{NativeComponent, NativeTemplateGrammar},
};
use vize_patina::{
    Linter,
    native::{NativeLintRefusal, NativeSyntaxLint},
};

#[test]
fn equal_foreign_owners_and_raw_parses_refuse_before_lookup_with_source_custody_retained() {
    let source = "<template><center /></template>";
    let arena = Allocator::default();
    let first = selected(&arena, source);
    let second = selected(&arena, source);
    let other_arena = Allocator::default();
    let third = selected(&other_arena, source);
    let raw = NativeComponent::parse_in(&arena, first.component().block()).unwrap();
    let lint = NativeSyntaxLint::new(&first).unwrap();
    for component in [second.component(), third.component(), &raw] {
        let element = component.children().next().unwrap().into_element().unwrap();
        assert_eq!(
            lint.deprecated_element(&element, &NeverLookup).err(),
            Some(NativeLintRefusal::ForeignElement)
        );
        assert!(core::ptr::eq(element.component(), component));
        assert_eq!(
            component.block().source(),
            first.component().block().source()
        );
        assert_eq!(check_fidelity(&component.carrier().tree), Ok(()));
    }
    assert!(core::ptr::eq(lint.owner(), &first));
    assert_eq!(check_fidelity(&first.component().carrier().tree), Ok(()));
}

#[test]
fn complete_nonmatching_and_component_headers_leave_the_catalog_unused() {
    for tag in ["Center", "CENTER", "Foo", "div", "widget", "x:center"] {
        let source = cstr!("<template><{tag} :title.prop='value' /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        assert!(
            lint.deprecated_element(&element, &NeverLookup)
                .unwrap()
                .is_none()
        );
        for locale in LOCALES {
            assert_eq!(complete(&registered(&source, locale)), expected(vec![]));
        }
    }
}

#[test]
fn owned_diagnostic_outlives_original_owner_arena_and_dynamic_message_provider() {
    struct OwnedCatalog {
        calls: Cell<usize>,
    }
    impl MessageLookup for OwnedCatalog {
        fn lookup(&self, key: &str) -> Cow<'static, str> {
            self.calls.set(self.calls.get() + 1);
            Cow::Owned(match key {
                "html/deprecated-element.message" => "owned {tag}/{tag} message".to_owned(),
                "html/deprecated-element.help" => "owned full help".to_owned(),
                _ => panic!("only the actual deprecated rule catalog keys are allowed"),
            })
        }
    }
    let (finding, before) = {
        let arena = Allocator::default();
        let source = "<template><center /></template>";
        let owner = selected(&arena, source);
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let messages = OwnedCatalog {
            calls: Cell::new(0),
        };
        let finding = lint
            .deprecated_element(&element, &messages)
            .unwrap()
            .unwrap();
        assert_eq!(messages.calls.get(), 2);
        assert_eq!(
            finding.diagnostic().message.as_str(),
            "owned center/center message"
        );
        let before = native(&finding);
        (finding, before)
    };
    assert_eq!(native(&finding), before);
    let diagnostic = finding.into_diagnostic();
    assert_eq!(diagnostic.message.as_str(), "owned center/center message");
    assert_eq!(diagnostic.parts[0].message.as_str(), "owned full help");
}

#[test]
fn original_selected_owner_can_move_after_borrows_end_without_reparse_or_profile_change() {
    let source = "<script setup lang=ts>const marker=1</script><template><center /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let grammar = owner.grammar();
    assert_eq!(grammar, NativeTemplateGrammar::TypeScriptModule);
    let source_ptr = owner.component().carrier().tree.source.as_ptr();
    let before = cstr!("{:?}", owner.component().carrier());
    {
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        assert!(
            lint.deprecated_element(&element, &messages(vize_patina::Locale::En))
                .unwrap()
                .is_some()
        );
    }
    let moved = owner;
    assert_eq!(moved.grammar(), grammar);
    assert_eq!(moved.component().carrier().tree.source.as_ptr(), source_ptr);
    assert_eq!(cstr!("{:?}", moved.component().carrier()), before);
    assert_eq!(check_fidelity(&moved.component().carrier().tree), Ok(()));
}

#[test]
fn a_truthful_supplied_header_does_not_claim_completion_of_unrelated_body_errors() {
    let source = "<template><center></center><div><span></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let element = owner.children().next().unwrap().into_element().unwrap();
    for locale in LOCALES {
        let finding = lint
            .deprecated_element(&element, &messages(locale))
            .unwrap()
            .unwrap();
        assert_eq!(
            native(&finding),
            warning(locale, span(source, "<center>"), "center")
        );
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                warning(locale, span(source, "<center>"), "center"),
                parser(
                    "error",
                    "Element is missing end tag.",
                    span(source, "<span>")
                ),
            ])
        );
    }
}

#[test]
fn opt_in_calls_preserve_complete_default_sfc_results_in_all_locales() {
    let source =
        "<!--🦀--><template><center title='x'><widget><blink /></widget></center></template>";
    for locale in LOCALES {
        let linter = Linter::default().with_locale(locale);
        let before = complete(&linter.lint_sfc(source, FILENAME));
        parity(source, locale);
        assert_eq!(complete(&linter.lint_sfc(source, FILENAME)), before);
    }
}
