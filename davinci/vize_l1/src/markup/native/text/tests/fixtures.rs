use super::{Allocator, SurfaceChild, selected};
use serde_json::Value;
use vize_l0::Span;

#[test]
fn every_pinned_original_root_text_matches_whole_source_qualified_golden_windows()
-> Result<(), &'static str> {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../../tests/fixtures/native-root-text-vue-3.5.35.json"
    ))
    .map_err(|_| "complete pinned packet")?;
    assert_eq!(
        fixture.get("vueVersion").and_then(Value::as_str),
        Some("3.5.35")
    );
    let cases = fixture
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("complete cases")?;
    assert_eq!(cases.len(), 37);
    let mut executed = 0;
    for case in cases {
        if case.get("nativeEligible").and_then(Value::as_bool) != Some(true) {
            continue;
        }
        executed += 1;
        let id = case
            .get("id")
            .and_then(Value::as_str)
            .ok_or("case identity")?;
        let source = case
            .get("source")
            .and_then(Value::as_str)
            .ok_or("whole original source")?;
        let arena = Allocator::default();
        let owner = selected(&arena, source)?;
        let children = owner.children().collect::<alloc::vec::Vec<_>>();
        let slots = case
            .get("rootSlots")
            .and_then(Value::as_array)
            .ok_or("complete original slots")?;
        assert_eq!(
            children.len(),
            slots.len(),
            "{id}: every original root event"
        );
        for (child, slot) in children.iter().zip(slots) {
            let ordinal = slot
                .get("ordinal")
                .and_then(Value::as_u64)
                .ok_or("original ordinal")?;
            assert_eq!(child.ordinal() as u64, ordinal, "{id}");
            let (kind, span) = original_span(&owner, child.surface())?;
            assert_eq!(slot.get("kind").and_then(Value::as_str), Some(kind), "{id}");
            assert_eq!(
                slot.get("span"),
                Some(&serde_json::json!([span.start, span.end])),
                "{id}"
            );
            assert_eq!(
                slot.get("raw").and_then(Value::as_str),
                Some(span.slice(source)),
                "{id}"
            );
        }
        let texts = case
            .get("rootText")
            .and_then(Value::as_array)
            .ok_or("complete original text windows")?;
        assert_eq!(
            texts.len(),
            children
                .iter()
                .filter(|child| matches!(child.surface(), SurfaceChild::Text(_)))
                .count(),
            "{id}: every original text window including omissions"
        );
        for text in texts {
            let ordinal = text
                .get("ordinal")
                .and_then(Value::as_u64)
                .and_then(|index| usize::try_from(index).ok())
                .ok_or("actual root ordinal")?;
            let child = children.get(ordinal).ok_or("actual original slot")?;
            let receipt = owner
                .prepare_condensed_root_text(child.reborrow())
                .map_err(|_| "native source window")?;
            assert_eq!(
                text.get("raw").and_then(Value::as_str),
                Some(receipt.raw_text()),
                "{id}"
            );
            assert_eq!(
                text.get("span"),
                Some(&serde_json::json!([
                    receipt.span().start,
                    receipt.span().end
                ])),
                "{id}"
            );
            assert_eq!(
                text.get("content"),
                Some(&serde_json::json!(receipt.content())),
                "{id}"
            );
            assert_eq!(receipt.span().slice(source), receipt.raw_text(), "{id}");
            assert!(
                receipt.admitted_for_root_at(&owner, ordinal).is_some(),
                "{id}"
            );
        }
        crate::check_fidelity(&owner.component().carrier().tree)
            .map_err(|_| "complete original source fidelity")?;
    }
    assert_eq!(executed, 28, "actual complete eligible cases");
    Ok(())
}

fn original_span(
    owner: &crate::markup::NativeTemplateComponent<'_>,
    child: &SurfaceChild<'_>,
) -> Result<(&'static str, Span), &'static str> {
    let block = owner.component().block();
    let span = |text| block.span_of(text).ok_or("actual authored source span");
    match child {
        SurfaceChild::Text(token) => Ok(("text", span(token.text)?)),
        SurfaceChild::Comment(token) => Ok(("comment", span(token.text)?)),
        SurfaceChild::Interpolation(interpolation) => Ok((
            "interpolation",
            Span::new(
                span(interpolation.open.text)?.start,
                span(interpolation.close.text)?.end,
            ),
        )),
        SurfaceChild::Element(element) => {
            let start = span(element.open.lt_name.text)?.start;
            let end = match &element.close {
                crate::ElementClose::Present(close) => span(close.gt.text)?.end,
                crate::ElementClose::NotExpected => span(element.open.gt.text)?.end,
                _ => return Err("complete original element frame"),
            };
            Ok(("element", Span::new(start, end)))
        }
        _ => Err("bounded original root kind"),
    }
}
