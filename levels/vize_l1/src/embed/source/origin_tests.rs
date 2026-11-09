use super::{
    DecodeMap, EmbedSource, SourceError, prepare_attribute_value, prepare_vue_interpolation_in,
};
use vize_l0::{Allocator, Span, String};

fn same_buffer(left: &str, right: &str) -> bool {
    left.as_ptr() == right.as_ptr() && left.len() == right.len()
}

#[test]
fn authored_and_decoded_sources_keep_the_complete_preparation_input() {
    let arena = Allocator::default();
    for input in ["xxαβyy", "xxα&fjlig;βyy", "xx&acE;yy"] {
        // Force distinct live heap buffers even for compact strings.
        let mut root = String::with_capacity(64);
        root.push_str(input);
        let mut equal_foreign = String::with_capacity(64);
        equal_foreign.push_str(input);
        assert_eq!(root, equal_foreign);
        assert!(!same_buffer(&root, &equal_foreign));
        let source =
            prepare_attribute_value(&arena, &root, Span::new(2, root.len() as u32 - 2)).unwrap();
        assert!(same_buffer(source.authored_root(), &root));
        assert!(!same_buffer(source.authored_root(), &equal_foreign));
        assert!(same_buffer(
            source
                .slice_in(&arena, Span::new(0, 0))
                .unwrap()
                .authored_root(),
            &root
        ));
        assert!(same_buffer(
            source
                .slice_in(&arena, Span::new(0, source.text().len() as u32))
                .unwrap()
                .authored_root(),
            &root
        ));
    }
}

#[test]
fn entity_and_identity_slices_keep_the_same_origin_and_exact_boundaries() {
    let arena = Allocator::default();
    let root = "xxα&fjlig;βyy";
    let source = prepare_attribute_value(&arena, root, Span::new(2, 13)).unwrap();
    assert_eq!(source.text(), "αfjβ");
    for relative in [Span::new(0, 2), Span::new(2, 4), Span::new(4, 6)] {
        let piece = source.slice_in(&arena, relative).unwrap();
        assert!(same_buffer(piece.authored_root(), root));
        assert_eq!(piece.span(), source.authored_span(relative).unwrap());
        assert_eq!(piece.decode_map().is_some(), relative == Span::new(2, 4));
    }
    assert!(matches!(
        source.slice_in(&arena, Span::new(2, 3)),
        Err(SourceError::PartialEntityBoundary)
    ));
    assert!(matches!(
        source.slice_in(&arena, Span::new(1, 2)),
        Err(SourceError::InvalidDecodedSpan)
    ));
}

#[test]
fn trimmed_interpolation_retains_whole_input_and_entity_spelling() {
    let arena = Allocator::default();
    let root = "xx \tα&amp;β\n yy";
    let source =
        prepare_vue_interpolation_in(&arena, root, Span::new(2, root.len() as u32 - 2)).unwrap();
    assert!(same_buffer(source.authored_root(), root));
    assert_eq!(source.text(), "α&β");
    assert_eq!(
        root.get(source.span().start as usize..source.span().end as usize),
        Some("α&amp;β")
    );
}

#[test]
fn shared_and_empty_buffers_do_not_create_document_identity() {
    for root in ["", "shared"] {
        let first = EmbedSource::authored(root, Span::new(0, root.len() as u32)).unwrap();
        let second = EmbedSource::authored(root, Span::new(0, root.len() as u32)).unwrap();
        assert!(same_buffer(first.authored_root(), second.authored_root()));
        // Equal origin says only that callers supplied the same borrowed bytes.
        assert_eq!(first.span(), second.span());
    }
}

#[test]
fn root_metadata_adds_one_borrow_without_plain_preparation_allocations() {
    assert!(!core::mem::needs_drop::<EmbedSource<'_>>());
    assert_eq!(
        core::mem::size_of::<EmbedSource<'_>>(),
        core::mem::size_of::<(Span, &str, Option<DecodeMap<'_>>)>() + core::mem::size_of::<&str>()
    );
    #[cfg(target_pointer_width = "64")]
    assert_eq!(core::mem::size_of::<EmbedSource<'_>>(), 56);
    let arena = Allocator::default();
    let root = "αβ";
    let source = prepare_attribute_value(&arena, root, Span::new(0, 4)).unwrap();
    assert!(same_buffer(
        source
            .slice_in(&arena, Span::new(0, 2))
            .unwrap()
            .authored_root(),
        root
    ));
    assert_eq!(arena.allocated_bytes(), 0);
}
