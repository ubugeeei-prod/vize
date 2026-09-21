use super::{Document, full_change, pos, ranged_change, test_uri};
use crate::utils::position_to_offset;

#[test]
fn incremental_edits_after_unicode_separators_use_the_authored_lsp_line() {
    let mut doc = Document::new(
        test_uri(),
        "a\u{2028}😀b\r\nc\u{2029}d".into(),
        1,
        "vue".into(),
    );
    assert_eq!(doc.line_count(), 2);
    doc.apply_change(&ranged_change(pos(0, 4), pos(0, 5), "Z"), 2);
    assert_eq!(doc.text(), "a\u{2028}😀Z\r\nc\u{2029}d");
    assert_eq!(doc.line(0).as_deref(), Some("a\u{2028}😀Z\r\n"));
    doc.apply_change(&ranged_change(pos(0, 1), pos(0, 2), ""), 3);
    assert_eq!(doc.text(), "a😀Z\r\nc\u{2029}d");
    doc.apply_change(&ranged_change(pos(1, 1), pos(1, 1), "\u{85}"), 4);
    assert_eq!(doc.text(), "a😀Z\r\nc\u{85}\u{2029}d");
    assert_eq!(doc.line_count(), 2);
    assert_eq!(
        position_to_offset(&doc.content, pos(1, 3)),
        Some(doc.text().len() - 1)
    );
    doc.apply_change(&full_change("clean\nsource"), 5);
    assert_eq!(doc.line_count(), 2);
    assert_eq!(position_to_offset(&doc.content, pos(1, 1)), Some(7));
}

#[test]
fn replacing_unicode_between_cr_and_lf_joins_the_lsp_line_boundary() {
    let mut doc = Document::new(test_uri(), "a\r\u{2028}\nb".into(), 1, "vue".into());
    assert_eq!(doc.line_count(), 3);
    doc.apply_change(&ranged_change(pos(1, 0), pos(1, 1), ""), 2);
    assert_eq!(doc.text(), "a\r\nb");
    assert_eq!(doc.line_count(), 2);
    doc.apply_change(&ranged_change(pos(1, 0), pos(1, 1), "done"), 3);
    assert_eq!(doc.text(), "a\r\ndone");
}
