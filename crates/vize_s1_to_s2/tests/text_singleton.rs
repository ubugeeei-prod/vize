//! Final text children keep leaf semantics without temporary compound parts.

mod support;

use vize_s0::Span;
use vize_s2::folio::{FolioExpr, FolioOp};

use support::{assert_transformed_sound, with_transformed};

#[test]
fn final_text_keeps_condensing_provenance_and_authored_spans() {
    for (source, raw, rendered) in [
        ("<p>日本語  😀</p>", "日本語  😀", "日本語 😀"),
        ("<p><i/>日本語\r\n  😀</p>", "日本語\r\n  😀", "日本語 😀"),
        (
            "<pre>日本語\r\n  😀</pre>",
            "日本語\r\n  😀",
            "日本語\r\n  😀",
        ),
    ] {
        with_transformed(source, |lowered, folio, facts, _| {
            let start = source.find(raw).unwrap() as u32;
            let span = Span::new(start, start + raw.len() as u32);
            let FolioOp::Element(element) = &folio.ops[0] else {
                panic!("expected an element");
            };
            assert!(matches!(element.children.last(), Some(FolioOp::Text(text))
                if text.content == rendered && text.span == span));
            assert!(lowered.texts.is_empty());
            assert!(facts.text_facts.is_empty());
            let mut records = lowered
                .provenance
                .iter()
                .filter(|record| record.span == span);
            if raw != rendered {
                let condensed = records.next().unwrap();
                assert_eq!(condensed.rule, "condense.whitespace");
                assert_eq!(condensed.before, raw);
                assert_eq!(condensed.after, rendered);
            }
            let leaf = records.next().unwrap();
            assert_eq!(leaf.rule, "lower.text");
            assert_eq!(leaf.before, raw);
            assert!(records.next().is_none());
        });
        assert_transformed_sound(source, "final-text");
    }
}

#[test]
fn final_interpolation_keeps_its_retained_expression_and_authored_span() {
    let source = "<p><i/>{{ 日本語 + 1 }}</p>";
    with_transformed(source, |lowered, folio, facts, _| {
        let FolioOp::Element(element) = &folio.ops[0] else {
            panic!("expected an element");
        };
        let Some(FolioOp::Interpolation(interpolation)) = element.children.last() else {
            panic!("expected a final interpolation");
        };
        assert_eq!(interpolation.span, Span::new(7, 26));
        assert!(matches!(&interpolation.expression,
            FolioExpr::Js { source, span, .. }
                if source == "日本語 + 1" && *span == Span::new(10, 23)));
        assert!(lowered.texts.is_empty());
        assert!(facts.text_facts.is_empty());
        assert_eq!(
            lowered
                .provenance
                .iter()
                .filter(|record| record.rule == "lower.interpolation"
                    && record.span == interpolation.span)
                .count(),
            1
        );
    });
    assert_transformed_sound(source, "final-interpolation");
}
