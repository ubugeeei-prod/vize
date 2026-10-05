use super::*;

#[test]
fn original_7502_text_has_exact_once_prepared_value_and_complete_authored_map() {
    // Immutable original regression 12, with only the real Descriptor envelope.
    // Its old six product lower refusals remain unchanged; this is L1 only.
    let arena = Allocator::default();
    let source = r#"<template><div title="a &amp;lt; b">&amp;lt;</div></template>"#;
    let owner = selected(&arena, source);
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let child = parent.children().next().unwrap();
    let original = child.surface();
    let value = owner.observe_text_value(child.reborrow()).unwrap();
    assert_eq!(value.raw_text(), "&amp;lt;");
    assert_eq!(value.source().text(), "&lt;");
    assert_eq!(value.span(), Span::new(36, 44));
    assert_eq!(value.span().slice(source), value.raw_text());
    assert!(core::ptr::eq(value.source().authored_root(), source));
    let segments = value.source().decode_map().unwrap().segments();
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].kind(), DecodeSegmentKind::Entity);
    assert_eq!(segments[0].decoded(), Span::new(0, 1));
    assert_eq!(segments[0].authored(), Span::new(36, 41));
    assert_eq!(segments[1].kind(), DecodeSegmentKind::Identity);
    assert_eq!(segments[1].decoded(), Span::new(1, 4));
    assert_eq!(segments[1].authored(), Span::new(41, 44));
    assert_eq!(
        value.source().authored_span(Span::new(0, 4)).unwrap(),
        value.span()
    );
    let view = value.admitted_for(&owner, child).unwrap();
    assert!(core::ptr::eq(view.child().surface(), original));
    assert!(core::ptr::eq(
        view.child().parent_element().unwrap(),
        parent.surface()
    ));
    assert!(core::ptr::eq(view.observation(), &value));
    crate::check_fidelity(&owner.component().carrier().tree).unwrap();
}

#[test]
fn unicode_numeric_multiscalar_and_unknown_atoms_keep_full_map_and_precise_projection() {
    let arena = Allocator::default();
    let source = "<!--前--><template><p>雪🌸 &acE; &#x1F338; &amp;lt; &unknown;</p></template>";
    let owner = selected(&arena, source);
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let value = owner
        .observe_text_value(parent.children().next().unwrap())
        .unwrap();
    assert_eq!(value.span(), Span::new(23, 65));
    assert_eq!(value.source().text(), "雪🌸 ∾̳ 🌸 &lt; &unknown;");
    assert_eq!(value.raw_text(), "雪🌸 &acE; &#x1F338; &amp;lt; &unknown;");
    let expected = [
        (
            Span::new(0, 8),
            Span::new(23, 31),
            DecodeSegmentKind::Identity,
        ),
        (
            Span::new(8, 13),
            Span::new(31, 36),
            DecodeSegmentKind::Entity,
        ),
        (
            Span::new(13, 14),
            Span::new(36, 37),
            DecodeSegmentKind::Identity,
        ),
        (
            Span::new(14, 18),
            Span::new(37, 46),
            DecodeSegmentKind::Entity,
        ),
        (
            Span::new(18, 19),
            Span::new(46, 47),
            DecodeSegmentKind::Identity,
        ),
        (
            Span::new(19, 20),
            Span::new(47, 52),
            DecodeSegmentKind::Entity,
        ),
        (
            Span::new(20, 33),
            Span::new(52, 65),
            DecodeSegmentKind::Identity,
        ),
    ];
    let actual = value.source().decode_map().unwrap().segments();
    assert_eq!(actual.len(), expected.len());
    for (segment, (decoded, authored, kind)) in actual.iter().zip(expected) {
        assert_eq!(segment.decoded(), decoded);
        assert_eq!(segment.authored(), authored);
        assert_eq!(segment.kind(), kind);
        assert_eq!(value.source().authored_span(decoded).unwrap(), authored);
        if kind == DecodeSegmentKind::Identity {
            assert_eq!(decoded.slice(value.source().text()), authored.slice(source));
        }
    }
    assert_eq!(
        value.source().authored_span(Span::new(8, 11)),
        Err(SourceError::PartialEntityBoundary)
    );
    assert_eq!(
        value
            .source()
            .authored_covering_span(Span::new(8, 11))
            .unwrap(),
        Span::new(31, 36)
    );
    assert_eq!(
        value.source().authored_span(Span::new(0, 33)).unwrap(),
        value.span()
    );
    crate::check_fidelity(&owner.component().carrier().tree).unwrap();
}

#[test]
fn text_context_differs_from_attribute_and_never_reparses_decoded_markup_or_interpolation() {
    let arena = Allocator::default();
    let source = "<template><p title='&amp=1 &timesX'>&amp=1 &timesX &lt;i&gt; &#123;&#123;x&#125;&#125;</p></template>";
    let owner = selected(&arena, source);
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let attr = owner
        .observe_attribute_value(parent.attributes().next().unwrap())
        .unwrap();
    assert_eq!(attr.source().text(), "&amp=1 &timesX");
    assert!(attr.source().decode_map().is_none());
    let value = owner
        .observe_text_value(parent.children().next().unwrap())
        .unwrap();
    assert_eq!(value.source().text(), "&=1 ×X <i> {{x}}");
    assert_eq!(parent.children().len(), 1);
    assert!(matches!(
        parent.children().next().unwrap().surface(),
        SurfaceChild::Text(_)
    ));
    assert!(value.source().decode_map().is_some());
    crate::check_fidelity(&owner.component().carrier().tree).unwrap();
}

#[test]
fn literal_and_unknown_values_borrow_without_preparation_allocation_or_whitespace_changes() {
    let arena = Allocator::default();
    for source in [
        "<template>雪🌸  \t\r\n text</template>",
        "<template><p>&unknown; a&b</p></template>",
        "<template><pre>\r\n  a\t b\r\n</pre></template>",
    ] {
        let owner = selected(&arena, source);
        let root = owner.children().next().unwrap();
        let parent = root.reborrow().into_element();
        let child = match &parent {
            Some(parent) => parent.children().next().unwrap(),
            None => root,
        };
        let before = arena.allocated_bytes();
        let value = owner.observe_text_value(child.reborrow()).unwrap();
        assert_eq!(arena.allocated_bytes(), before);
        assert!(value.source().decode_map().is_none());
        assert!(core::ptr::eq(value.raw_text(), value.source().text()));
        assert_eq!(value.raw_text(), value.span().slice(source));
        assert!(value.admitted_for(&owner, child).is_some());
        crate::check_fidelity(&owner.component().carrier().tree).unwrap();
    }
    let owner = selected(&arena, "<template> &#32;&#9;&NewLine;\r\n </template>");
    let value = owner
        .observe_text_value(owner.children().next().unwrap())
        .unwrap();
    assert_eq!(value.source().text(), "  \t\n\r\n ");
    assert_eq!(value.raw_text(), " &#32;&#9;&NewLine;\r\n ");
    assert!(value.source().decode_map().is_some());
}

#[test]
fn pre_namespace_and_table_source_preparation_grants_no_whitespace_or_element_policy() {
    let arena = Allocator::default();
    for (source, depth) in [
        ("<template><pre>\r\n &amp;lt;\t </pre></template>", 1),
        (
            "<template><svg><text>\r\n &amp;lt;\t </text></svg></template>",
            2,
        ),
        (
            "<template><table><tbody><tr><td>\r\n &amp;lt;\t </td></tr></tbody></table></template>",
            4,
        ),
    ] {
        let owner = selected(&arena, source);
        let mut child = owner.children().next().unwrap();
        // Only this test navigates the actual original parent chain.
        for _ in 0..depth {
            let parent = child.into_element().unwrap();
            child = parent.children().next().unwrap();
        }
        let value = owner.observe_text_value(child.reborrow()).unwrap();
        assert_eq!(value.raw_text(), "\r\n &amp;lt;\t ");
        assert_eq!(value.source().text(), "\r\n &lt;\t ");
        assert!(value.admitted_for(&owner, child).is_some());
        crate::check_fidelity(&owner.component().carrier().tree).unwrap();
    }
}
