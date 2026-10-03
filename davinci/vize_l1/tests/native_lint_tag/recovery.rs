use super::*;

#[test]
fn literal_receipts_capture_actual_header_entry_before_interactive_recovery() {
    for tag in ["a", "button"] {
        let source = cstr!(
            "<template><{tag} v-pre><{tag} :role='x'></{tag}></{tag}><meta role /></template>"
        );
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let outer = owner.children().next().unwrap().into_element().unwrap();
        assert!(matches!(outer.surface().close, ElementClose::Implicit));
        assert!(!outer.lint_tag().unwrap().header_is_literal());
        let inner = owner.children().nth(1).unwrap().into_element().unwrap();
        let receipt = inner.lint_tag().unwrap();
        assert!(receipt.header_is_literal());
        assert!(receipt.in_recovery_context());
        assert!(!inner.surface().open.is_verbatim());
        assert_eq!(
            inner.attributes().next().unwrap().surface().name.text,
            ":role"
        );
        let last = owner
            .children()
            .filter_map(|child| child.into_element())
            .last()
            .unwrap();
        assert_eq!(last.surface().tag(), "meta");
        assert!(!last.lint_tag().unwrap().header_is_literal());
        assert!(!last.lint_tag().unwrap().in_recovery_context());
        assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
    }
}

#[test]
fn original_recovery_ancestry_is_conservative_without_legacy_tree_substitution() {
    for tag in [
        "p", "form", "a", "button", "li", "dt", "dd", "option", "optgroup", "b", "big", "code",
        "em", "font", "i", "nobr", "s", "small", "strike", "strong", "tt", "u",
    ] {
        let source =
            cstr!("<template><{tag} v-pre><meta :role='x' /></{tag}><meta role /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let parent = owner.children().next().unwrap().into_element().unwrap();
        assert!(!parent.lint_tag().unwrap().in_recovery_context());
        let meta = parent.children().next().unwrap().into_element().unwrap();
        let receipt = meta.lint_tag().unwrap();
        assert!(receipt.header_is_literal());
        assert!(receipt.in_recovery_context());
        assert!(core::ptr::eq(receipt.element(), meta.surface()));
        let outside = owner.children().nth(1).unwrap().into_element().unwrap();
        assert!(!outside.lint_tag().unwrap().in_recovery_context());
        assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
    }
}

#[test]
fn self_closing_nested_interactive_headers_retain_the_actual_original_scope() {
    for tag in ["a", "button"] {
        let source = cstr!("<template><{tag} v-pre><{tag}/><meta :role='x' /></{tag}></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let parent = owner.children().next().unwrap().into_element().unwrap();
        let inner = parent.children().next().unwrap().into_element().unwrap();
        assert!(inner.lint_tag().unwrap().header_is_literal());
        assert!(inner.lint_tag().unwrap().in_recovery_context());
        let meta = parent.children().nth(1).unwrap().into_element().unwrap();
        assert!(meta.lint_tag().unwrap().header_is_literal());
        assert!(meta.lint_tag().unwrap().in_recovery_context());
        assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
    }
}

#[test]
fn genuine_original_depth_is_bounded_before_over_limit_pre_can_issue_facts() {
    let source = cstr!(
        "<template>{}<div v-pre><meta :role='x' /></div>{}</template>",
        "<div>".repeat(4096),
        "</div>".repeat(4096)
    );
    let arena = Allocator::default();
    let owner = selected(&arena, &source);
    let mut original = owner.children().next().unwrap().into_element().unwrap();
    for _ in 0..4096 {
        original = original.children().next().unwrap().into_element().unwrap();
    }
    assert_eq!(original.surface().tag(), "div");
    assert!(original.lint_tag().unwrap().in_recovery_context());
    let meta = original.children().next().unwrap().into_element().unwrap();
    let receipt = meta.lint_tag().unwrap();
    assert_eq!(receipt.kind(), Kind::Element);
    assert!(receipt.header_is_literal());
    assert!(receipt.in_recovery_context());
    assert!(core::ptr::eq(receipt.component(), owner.component()));
    assert!(core::ptr::eq(receipt.element(), meta.surface()));
    assert_eq!(
        receipt.span().start,
        u32::try_from(source.find("<meta").unwrap() + 1).unwrap()
    );
}
