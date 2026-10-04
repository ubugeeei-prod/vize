use super::*;
use crate::container::Vue;
use crate::container::vue::DescriptorOptions;
use crate::embed::DecodeSegmentKind;
use vize_l0::Allocator;
use vize_l0::config::{VueDialect, VueVersion};

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: crate::SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

#[test]
fn all_fourteen_original_7502_attribute_inputs_keep_once_decoded_values() {
    // The authentic Descriptor envelope preserves each legacy template verbatim.
    // This is source preparation evidence, not a File or Vapor runtime claim.
    let cases = [
        (r#"<div title="a &amp;lt; b">x</div>"#, "a &lt; b"),
        (r#"<div title="&amp;lt;">x</div>"#, "&lt;"),
        (r#"<div title="&#38;copy;">x</div>"#, "&copy;"),
        (r#"<div title="&#x26;#60;">x</div>"#, "&#60;"),
        (r#"<div title="&amp;amp;lt;">x</div>"#, "&amp;lt;"),
        (r#"<div title="a&b">x</div>"#, "a&b"),
        (r#"<div title='a&amp;lt;"b'>x</div>"#, "a&lt;\"b"),
        (r#"<div title='a"b'>x</div>"#, "a\"b"),
        ("<div title=a&b>x</div>", "a&b"),
        (r#"<div title="&lt;b&gt;">x</div>"#, "<b>"),
        ("<div title=&amp;lt;>x</div>", "&lt;"),
        (r#"<div class="a&amp;amp;b">x</div>"#, "a&amp;b"),
        (r#"<div title="a &amp;lt; b">&amp;lt;</div>"#, "a &lt; b"),
        (r#"<div title="plain">x</div>"#, "plain"),
    ];
    for (original, decoded) in cases {
        let arena = Allocator::default();
        let source = alloc::format!("<template>{original}</template>");
        let selected = selected(&arena, &source);
        assert_eq!(selected.component().block().source(), original);
        let element = selected.children().next().unwrap().into_element().unwrap();
        let mut visits = 0;
        for attribute in element.attributes() {
            visits += 1;
            let observed = selected
                .observe_attribute_value(attribute.reborrow())
                .unwrap();
            assert_eq!(observed.source().text(), decoded, "{original}");
            assert!(core::ptr::eq(
                observed.source().authored_root(),
                source.as_str()
            ));
            assert_eq!(observed.value_span().slice(&source), observed.raw_value());
            assert_eq!(observed.source().span(), observed.value_span());
            let actual = observed.admitted_for(&selected, attribute).unwrap();
            assert!(core::ptr::eq(actual.observation(), &observed));
            assert!(core::ptr::eq(
                actual.attribute().element(),
                element.surface()
            ));
        }
        assert_eq!(visits, 1);
    }
}

#[test]
fn unicode_nonzero_source_and_complete_entity_map_never_redecode_output() {
    let arena = Allocator::default();
    let source =
        "前\r\n<template><div title='雪🌸 &acE; &#x1F338; &amp;lt; &unknown;'/></template>";
    let selected = selected(&arena, source);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let observed = selected
        .observe_attribute_value(element.attributes().next().unwrap())
        .unwrap();
    assert_eq!(observed.source().text(), "雪🌸 ∾̳ 🌸 &lt; &unknown;");
    assert_eq!(observed.name_span().slice(source), "title");
    assert_eq!(observed.equals_span().slice(source), "=");
    assert_eq!(
        observed.full_value_span().slice(source),
        "'雪🌸 &acE; &#x1F338; &amp;lt; &unknown;'"
    );
    let (open, close) = observed.quote_spans().unwrap();
    assert_eq!(open.slice(source), "'");
    assert_eq!(close.slice(source), "'");
    assert_eq!(open.end, observed.value_span().start);
    assert_eq!(close.start, observed.value_span().end);
    let map = observed.source().decode_map().unwrap();
    let mut decoded = 0;
    let mut authored = observed.value_span().start;
    for segment in map.segments() {
        assert_eq!(segment.decoded().start, decoded);
        assert_eq!(segment.authored().start, authored);
        assert_eq!(
            observed.source().authored_span(segment.decoded()).unwrap(),
            segment.authored()
        );
        if segment.kind() == DecodeSegmentKind::Identity {
            assert_eq!(
                segment.decoded().slice(observed.source().text()),
                segment.authored().slice(source)
            );
        } else {
            assert!(segment.authored().slice(source).starts_with('&'));
        }
        decoded = segment.decoded().end;
        authored = segment.authored().end;
    }
    assert_eq!(decoded as usize, observed.source().text().len());
    assert_eq!(authored, observed.value_span().end);
    let multiscalar = map
        .segments()
        .iter()
        .find(|segment| segment.authored().slice(source) == "&acE;")
        .unwrap();
    assert_eq!(
        observed.source().authored_span(Span::new(
            multiscalar.decoded().start,
            multiscalar.decoded().start + 3
        )),
        Err(crate::embed::SourceError::PartialEntityBoundary)
    );
    assert_eq!(
        observed
            .source()
            .authored_covering_span(Span::new(
                multiscalar.decoded().start,
                multiscalar.decoded().start + 3
            ))
            .unwrap(),
        multiscalar.authored()
    );
}

#[test]
fn plain_unknown_empty_and_unquoted_values_borrow_without_preparation_allocation() {
    let arena = Allocator::default();
    let source = "<template><div title='plain' id=&unknown; role=button aria-label=\"\" data-x='a&b' /></template>";
    let selected = selected(&arena, source);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let mut visits = 0;
    for attribute in element.attributes() {
        visits += 1;
        let before = arena.allocated_bytes();
        let observed = selected
            .observe_attribute_value(attribute.reborrow())
            .unwrap();
        assert_eq!(arena.allocated_bytes(), before);
        assert!(observed.source().decode_map().is_none());
        assert!(core::ptr::eq(
            observed.raw_value(),
            observed.source().text()
        ));
        assert_eq!(
            observed.source().text(),
            attribute.surface().value.as_ref().unwrap().content.text
        );
        assert!(observed.admitted_for(&selected, attribute).is_some());
    }
    assert_eq!(visits, 5);
}

#[test]
fn single_header_visit_parks_and_moves_the_original_value_and_map_unchanged() {
    let arena = Allocator::default();
    let source = "<template><div title='a &amp;lt; b'/></template>";
    let selected = selected(&arena, source);
    let element = selected.children().next().unwrap().into_element().unwrap();
    let mut parked = alloc::vec::Vec::new();
    let mut visits = 0;
    for attribute in element.attributes() {
        visits += 1;
        let observed = selected
            .observe_attribute_value(attribute.reborrow())
            .unwrap();
        let text = observed.source().text();
        let segments = observed.source().decode_map().unwrap().segments();
        parked.push(observed);
        parked.reserve(32);
        let before = arena.allocated_bytes();
        let view = parked[0].admitted_for(&selected, attribute).unwrap();
        assert!(core::ptr::eq(view.observation().source().text(), text));
        assert!(core::ptr::eq(
            view.observation().source().decode_map().unwrap().segments(),
            segments
        ));
        assert_eq!(arena.allocated_bytes(), before);
    }
    assert_eq!(visits, 1);
    let selected = core::hint::black_box(selected);
    let element = selected.children().next().unwrap().into_element().unwrap();
    assert!(
        parked[0]
            .admitted_for(&selected, element.attributes().next().unwrap())
            .is_some()
    );
}

#[test]
fn preparation_is_source_only_and_preserves_intrinsic_selection_without_js_parse() {
    let arena = Allocator::default();
    for source in [
        "<template><div :id='not { javascript &amp;' class='a&amp;b'/></template>",
        "<template><div :id='not { javascript &amp;' class='a&amp;b'/></template><script setup lang=ts>const n=1</script>",
    ] {
        let selected = selected(&arena, source);
        let element = selected.children().next().unwrap().into_element().unwrap();
        for attribute in element.attributes() {
            let observed = selected
                .observe_attribute_value(attribute.reborrow())
                .unwrap();
            let view = observed.admitted_for(&selected, attribute).unwrap();
            assert_eq!(view.selected().grammar(), selected.grammar());
            assert!(matches!(
                observed.source().text(),
                "not { javascript &" | "a&b"
            ));
        }
    }
}

mod refusal;
