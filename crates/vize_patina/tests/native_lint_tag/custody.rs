use super::*;

#[test]
fn duplicate_attribute_advisories_remain_in_the_complete_registered_output() {
    let source = "<!--🦀--><template><div autofocus autofocus></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    assert!(owner.component().carrier().errors.is_empty());
    let div = owner.children().next().unwrap().into_element().unwrap();
    assert_eq!(div.surface().open.attrs.len(), 2);
    assert_eq!(div.lint_tag().unwrap().kind(), NativeLintTagKind::Element);
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        let (_, result) = observed(source, locale, &owner);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 3);
        assert_eq!(result.diagnostics.len(), 3);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_name == "parser/template")
        );
    }
}

#[test]
fn non_void_self_closing_control_keeps_the_registered_notice_suppression() {
    let source = "<template><div autofocus /></template>";
    let (output, result) = parity(source, Locale::En);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].kind, MarkupElementKind::Element);
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "parser/template")
    );
}

#[test]
fn selected_prefix_offsets_and_native_reads_leave_ordinary_routes_unchanged() {
    let source = "<!--🦀--><script lang=ts>const fake='<input autofocus />'</script><template>日本語<div><input autofocus /><Foo autofocus/></div></template><script setup lang=ts>const other='<slot autofocus />'</script>";
    let before = Linter::new().lint_sfc(source, "native.vue");
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        let (output, result) = parity(source, locale);
        assert_eq!(output.len(), 3);
        let start = u32::try_from(source.find("<input autofocus /><Foo").unwrap()).unwrap();
        assert_eq!(output[1].tag_span, Span::new(start + 1, start + 6));
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics[0].start, start + 7);
        assert_eq!(result.diagnostics[0].end, start + 16);
    }
    let after = Linter::new().lint_sfc(source, "native.vue");
    assert_eq!(complete_result(&before), complete_result(&after));
}

#[test]
fn equal_source_foreign_receipts_cannot_be_used_as_the_selected_lint_owner() {
    struct UnusedMessages;
    impl vize_l0::diag::MessageLookup for UnusedMessages {
        fn lookup(&self, _key: &str) -> std::borrow::Cow<'static, str> {
            panic!("foreign owner must refuse before any diagnostic lookup");
        }
    }
    let source = "<template><input autofocus /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let foreign = selected(&arena, source);
    let element = foreign.children().next().unwrap().into_element().unwrap();
    let receipt = element.lint_tag().unwrap();
    assert!(core::ptr::eq(receipt.component(), foreign.component()));
    assert!(!core::ptr::eq(receipt.component(), owner.component()));
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    assert!(matches!(
        lint.img_alt(&element, &UnusedMessages),
        Err(NativeLintRefusal::ForeignElement)
    ));
    assert!(matches!(
        lint.iframe_has_title(&element, &UnusedMessages),
        Err(NativeLintRefusal::ForeignElement)
    ));
    assert!(matches!(
        lint.tabindex_no_positive(&element, &UnusedMessages),
        Err(NativeLintRefusal::ForeignElement)
    ));
    assert_eq!(check_fidelity(&foreign.component().carrier().tree), Ok(()));
}
