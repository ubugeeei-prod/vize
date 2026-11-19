use super::*;

#[test]
fn original_special_parent_names_and_inherited_verbatim_refuse_before_preparation() {
    let arena = Allocator::default();
    for (source, expected) in [
        (
            "<template><script>&amp;</script></template>",
            TextValueError::RawTextParent,
        ),
        (
            "<template><stYle>&amp;</stYle></template>",
            TextValueError::RawTextParent,
        ),
        // Uppercase first/second letters are conservative authored-name
        // exclusions, not a claim of the actual lexer's special-mode entry.
        (
            "<template><sTyLe>&amp;</sTyLe></template>",
            TextValueError::RawTextParent,
        ),
        (
            "<template><StYlE>&amp;</StYlE></template>",
            TextValueError::RawTextParent,
        ),
        (
            "<template><title>&amp;</title></template>",
            TextValueError::RcDataParent,
        ),
        (
            "<template><teXtArEa>&amp;</teXtArEa></template>",
            TextValueError::RcDataParent,
        ),
        (
            "<template><tExTaReA>&amp;</tExTaReA></template>",
            TextValueError::RcDataParent,
        ),
        (
            "<template><TEXTAREA>&amp;</TEXTAREA></template>",
            TextValueError::RcDataParent,
        ),
        (
            "<template><p v-pre>&amp;</p></template>",
            TextValueError::Verbatim,
        ),
    ] {
        let owner = selected(&arena, source);
        let parent = owner.children().next().unwrap().into_element().unwrap();
        let child = parent.children().next().unwrap();
        assert!(matches!(child.surface(), SurfaceChild::Text(_)));
        let before = arena.allocated_bytes();
        let failure = owner.observe_text_value(child).err().unwrap();
        assert_eq!(failure.kind(), expected, "{source}");
        assert_eq!(arena.allocated_bytes(), before, "{source}");
        assert_eq!(failure.token().unwrap().text, "&amp;");
        assert_eq!(failure.span().unwrap().slice(source), "&amp;");
        assert_eq!(failure.parent_tag(), Some(parent.surface().tag()));
        assert!(core::ptr::eq(failure.block().root_source(), source));
        crate::markup::check_fidelity(&owner.component().carrier().tree).unwrap();
    }
    let owner = selected(
        &arena,
        "<template><div v-pre><span>&amp;</span></div></template>",
    );
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let descendant = parent.children().next().unwrap().into_element().unwrap();
    assert!(descendant.surface().open.is_verbatim());
    let failure = owner
        .observe_text_value(descendant.children().next().unwrap())
        .err()
        .unwrap();
    assert_eq!(failure.kind(), TextValueError::Verbatim);
    assert_eq!(failure.parent_tag(), Some("span"));
    assert_eq!(failure.token().unwrap().text, "&amp;");
}

#[test]
fn actual_nontext_events_keep_original_kind_instead_of_preparing_their_content() {
    let arena = Allocator::default();
    let source = "<template><!--kept-->{{ value }}<i/><![CDATA[&amp;]]><?kept?></bad></template>";
    let owner = selected(&arena, source);
    let before = arena.allocated_bytes();
    let mut visits = 0;
    for child in owner.children() {
        assert!(!matches!(child.surface(), SurfaceChild::Text(_)));
        let ordinal = child.ordinal();
        let failure = owner.observe_text_value(child).err().unwrap();
        assert_eq!(failure.kind(), TextValueError::NotText);
        assert_eq!(failure.ordinal(), ordinal);
        assert!(failure.token().is_none());
        assert!(failure.span().is_none());
        assert!(core::ptr::eq(failure.block().root_source(), source));
        visits += 1;
    }
    assert_eq!(visits, 6);
    assert_eq!(arena.allocated_bytes(), before);
    crate::markup::check_fidelity(&owner.component().carrier().tree).unwrap();
}

#[test]
fn genuine_missing_parent_close_keeps_full_raw_token_even_without_diagnostic_rows() {
    let arena = Allocator::default();
    let source = "<template><p>&amp;</template>";
    let owner = selected(&arena, source);
    let parent = owner.children().next().unwrap().into_element().unwrap();
    assert!(owner.component().carrier().errors.is_empty());
    assert!(owner.component().carrier().unsupported.is_empty());
    assert!(matches!(
        parent.surface().close,
        crate::markup::ElementClose::Missing
    ));
    let before = arena.allocated_bytes();
    let failure = owner
        .observe_text_value(parent.children().next().unwrap())
        .err()
        .unwrap();
    assert_eq!(failure.kind(), TextValueError::RecoveredComponent);
    assert_eq!(arena.allocated_bytes(), before);
    assert_eq!(
        failure.token().unwrap().status,
        crate::markup::TokenStatus::Present
    );
    assert_eq!(failure.token().unwrap().text, "&amp;");
    assert_eq!(failure.span(), Some(Span::new(13, 18)));
    assert!(core::ptr::eq(failure.block().root_source(), source));
    crate::markup::check_fidelity(&owner.component().carrier().tree).unwrap();
}

#[test]
fn long_vue_directive_arguments_preserve_original_decoded_prefix_text() {
    let arena = Allocator::default();
    let source = "<template>&amp;<div v-pre:[([([([([([([([([([([([([([([([([([([([([([([([([([([([([([([([([key])])])])])])])])])])])])])])])])])])])])])])])])])])])])])])])])]>body</div></template>";
    let owner = selected(&arena, source);
    assert_eq!(owner.component().carrier().unsupported.len(), 0);
    let child = owner.children().next().unwrap();
    let original = child.surface();
    let before = arena.allocated_bytes();
    let value = owner.observe_text_value(child).unwrap();
    assert_eq!(arena.allocated_bytes() - before, 21);
    assert_eq!(value.raw_text(), "&amp;");
    assert_eq!(value.source().text(), "&");
    assert_eq!(value.span(), Span::new(10, 15));
    assert!(core::ptr::eq(value.source().authored_root(), source));
    assert!(core::ptr::eq(
        owner.children().next().unwrap().surface(),
        original
    ));
    assert_eq!(owner.component().carrier().unsupported.len(), 0);
    crate::markup::check_fidelity(&owner.component().carrier().tree).unwrap();
}

#[test]
fn source_value_never_lifts_existing_root_condense_entity_or_nested_refusals() {
    let arena = Allocator::default();
    let owner = selected(&arena, "<template>&amp;lt;</template>");
    let child = owner.children().next().unwrap();
    let value = owner.observe_text_value(child.reborrow()).unwrap();
    assert_eq!(value.source().text(), "&lt;");
    assert!(matches!(
        owner.prepare_condensed_root_text(child),
        Err(RootTextError::Entity)
    ));
    let owner = selected(&arena, "<template><p>&amp;lt;</p></template>");
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let child = parent.children().next().unwrap();
    let value = owner.observe_text_value(child.reborrow()).unwrap();
    assert!(value.admitted_for(&owner, child.reborrow()).is_some());
    assert!(matches!(
        owner.prepare_condensed_root_text(child),
        Err(RootTextError::NestedChild)
    ));
    crate::markup::check_fidelity(&owner.component().carrier().tree).unwrap();
}
