use super::*;
use vize_l0::SourceRoot;

#[test]
fn annotation_preflight_refuses_overlap_order_empty_foreign_and_utf8_boundaries()
-> Result<(), &'static str> {
    let root = "前|abc雪def|後";
    let start = root.find("abc").ok_or("actual block")?;
    let text = root
        .get(start..start + "abc雪def".len())
        .ok_or("block slice")?;
    let source = SourceRoot::new(root)
        .map_err(|_| "root")?
        .block(text, start as u32)
        .map_err(|_| "same root slice")?;
    let at = source.start();
    let first = Span::new(at, at + 3);
    let last = Span::new(at + 6, at + 9);
    assert_eq!(validate_spans(source, [first, last].into_iter()), Ok(()));
    for (spans, rejected) in [
        (
            vec![first, Span::new(at + 2, at + 3)],
            Span::new(at + 2, at + 3),
        ),
        (vec![last, first], first),
        (vec![Span::new(at + 3, at + 3)], Span::new(at + 3, at + 3)),
        (vec![Span::new(at + 6, at + 3)], Span::new(at + 6, at + 3)),
        (vec![Span::new(0, 3)], Span::new(0, 3)),
        (vec![Span::new(at + 9, at + 10)], Span::new(at + 9, at + 10)),
        (vec![Span::new(at + 3, at + 4)], Span::new(at + 3, at + 4)),
        (vec![Span::new(at + 4, at + 6)], Span::new(at + 4, at + 6)),
    ] {
        assert_eq!(
            validate_spans(source, spans.into_iter()),
            Err(invalid(rejected))
        );
    }
    Ok(())
}
