use super::*;

#[test]
fn absolute_original_tag_spans_keep_selected_roles_and_unicode_prefixes() {
    let source = "<!--🦀--><script lang=ts>const fake='<INPUT autofocus />'</script><template>日本語<div><input autofocus /><AÉFoo/></div></template><script setup lang=ts>const other='<slot />'</script>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    assert_eq!(owner.grammar(), NativeTemplateGrammar::TypeScriptModule);
    assert!(core::ptr::eq(
        owner.component().block().root_source(),
        source
    ));
    assert_eq!(
        tags(&owner),
        [
            ("div", Ok(Kind::Element)),
            ("input", Ok(Kind::Element)),
            ("AÉFoo", Ok(Kind::Component)),
        ]
    );
    let div = owner.children().nth(1).unwrap().into_element().unwrap();
    let input = div.children().next().unwrap().into_element().unwrap();
    let receipt = input.lint_tag().unwrap();
    let start = u32::try_from(source.find("<input autofocus").unwrap() + 1).unwrap();
    assert_eq!(receipt.span(), Span::new(start, start + 5));
    assert_eq!(
        input.attributes().next().unwrap().surface().name.text,
        "autofocus"
    );
}

#[test]
fn equal_sources_keep_distinct_owners_and_short_receipts_survive_reborrowing() {
    let source = "<template><Foo/><input autofocus /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let foreign = selected(&arena, source);
    let original_surface = {
        let first = owner.children().next().unwrap().into_element().unwrap();
        let other = foreign.children().next().unwrap().into_element().unwrap();
        let receipt = first.lint_tag().unwrap();
        let other_receipt = other.lint_tag().unwrap();
        assert_eq!(receipt.kind(), other_receipt.kind());
        assert_eq!(receipt.span(), other_receipt.span());
        assert!(!core::ptr::eq(
            receipt.component(),
            other_receipt.component()
        ));
        assert!(!core::ptr::eq(receipt.element(), other_receipt.element()));
        assert!(core::ptr::eq(receipt.component(), owner.component()));
        receipt.element() as *const Element<'_>
    };
    let moved = core::hint::black_box(owner);
    let first = moved.children().next().unwrap().into_element().unwrap();
    let receipt = first.lint_tag().unwrap();
    assert!(core::ptr::eq(receipt.element(), original_surface));
    assert!(core::ptr::eq(receipt.component(), moved.component()));
    assert_eq!(
        tags(&moved),
        [("Foo", Ok(Kind::Component)), ("input", Ok(Kind::Element))]
    );
}

#[test]
fn arbitrary_native_blocks_and_raw_compatibility_construction_cannot_mint_selected_facts() {
    let source = "<template><div v-pre><Foo/></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let block = owner.component().block();
    let arbitrary = NativeComponent::parse_in(&arena, block).unwrap();
    let arbitrary_div = arbitrary.children().next().unwrap().into_element().unwrap();
    assert!(matches!(
        arbitrary_div.lint_tag(),
        Err(Refusal::Unavailable)
    ));
    assert_eq!(arbitrary.block(), block);
    assert_eq!(
        rendered(&arbitrary.carrier().tree),
        rendered(&owner.component().carrier().tree)
    );
    let compat = vize_l1::parse(&arena, block.source());
    assert_eq!(
        cstr!("{:?}", owner.component().carrier().tree.children),
        cstr!("{:?}", compat.0.children)
    );
    let raw = OpenTag {
        lt_name: Token::present("", "<div"),
        attrs: vize_l0::Vec::new_in(&&arena),
        slash: None,
        gt: Token::present("", ">"),
    };
    assert!(!raw.is_verbatim());
    assert_eq!(
        cstr!("{:?}", raw.lt_name),
        "Token { leading: \"\", text: \"<div\", status: Present }"
    );
    if cfg!(target_pointer_width = "64") {
        assert_eq!(core::mem::size_of::<Token<'_>>(), 40);
        assert_eq!(core::mem::size_of::<Option<Token<'_>>>(), 40);
        assert_eq!(core::mem::size_of::<OpenTag<'_>>(), 144);
        assert_eq!(core::mem::size_of::<Element<'_>>(), 248);
    }
}

#[test]
fn incomplete_headers_refuse_without_erasing_recovery_or_claiming_body_completion() {
    let source = "<template><div title='unfinished</template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let div = owner.children().next().unwrap().into_element().unwrap();
    assert!(matches!(div.lint_tag(), Err(Refusal::IncompleteHeader)));
    assert!(div.surface().open.gt.is_missing());
    assert!(!owner.component().carrier().errors.is_empty());
    assert_eq!(owner.component().block().source(), "<div title='unfinished");
    assert_eq!(
        rendered(&owner.component().carrier().tree).as_str(),
        owner.component().block().source()
    );
    let arbitrary =
        NativeComponent::parse_in(&arena, SourceRoot::new("<div").unwrap().whole_block()).unwrap();
    let incomplete = arbitrary.children().next().unwrap().into_element().unwrap();
    assert!(matches!(
        incomplete.lint_tag(),
        Err(Refusal::IncompleteHeader)
    ));
    let complete_header = selected(&arena, "<template><div></template>");
    let div = complete_header
        .children()
        .next()
        .unwrap()
        .into_element()
        .unwrap();
    assert!(matches!(div.surface().close, ElementClose::Missing));
    assert_eq!(div.lint_tag().unwrap().kind(), Kind::Element);
    assert_eq!(
        check_fidelity(&complete_header.component().carrier().tree),
        Ok(())
    );
}

#[test]
fn live_recovery_ends_ambiguous_scope_without_rewriting_original_observations() {
    let source =
        "<template><a v-pre.foo><span><a></a></span></a><input :autofocus='outside' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let arbitrary = NativeComponent::parse_in(&arena, owner.component().block()).unwrap();
    assert_eq!(
        cstr!("{:?}", owner.component().carrier().tree.children),
        cstr!("{:?}", arbitrary.carrier().tree.children)
    );
    assert_eq!(
        cstr!("{:?}", owner.component().carrier().errors),
        cstr!("{:?}", arbitrary.carrier().errors)
    );
    assert_eq!(
        tags(&owner),
        [
            ("a", Err(Refusal::AmbiguousVerbatim)),
            ("span", Err(Refusal::AmbiguousVerbatim)),
            ("a", Ok(Kind::Element)),
            ("input", Ok(Kind::Element)),
        ]
    );
    let outer = owner.children().next().unwrap().into_element().unwrap();
    assert!(matches!(outer.surface().close, ElementClose::Implicit));
    assert!(outer.surface().open.is_verbatim());
    let inner = owner.children().nth(1).unwrap().into_element().unwrap();
    assert!(!inner.surface().open.is_verbatim());
    assert_eq!(
        rendered(&owner.component().carrier().tree).as_str(),
        owner.component().block().source()
    );
}

#[test]
fn non_ascii_initial_tag_bytes_remain_text_without_fabricated_elements() {
    let arena = Allocator::default();
    let owner = selected(&arena, "<template><ÉFoo /><éfoo /></template>");
    assert!(tags(&owner).is_empty());
    let ascii_initial = selected(&arena, "<template><AÉFoo/><aÉfoo></aÉfoo></template>");
    assert_eq!(
        tags(&ascii_initial),
        [("AÉFoo", Ok(Kind::Component)), ("aÉfoo", Ok(Kind::Element))]
    );
}
