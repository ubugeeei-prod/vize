use super::Buffer;
use crate::{
    layout::Rect,
    terminal::cell::{Cell, Style},
};

#[test]
fn creates_sets_and_resizes_buffers() {
    let mut buffer = Buffer::new(10, 10);
    assert_eq!(buffer.area(), Rect::new(0, 0, 10, 10));

    buffer.set(5, 5, Cell::new("A"));
    assert_eq!(buffer.get(5, 5).map(|cell| cell.symbol.as_str()), Some("A"));

    buffer.resize(20, 20);
    assert_eq!(buffer.area(), Rect::new(0, 0, 20, 20));
    assert_eq!(buffer.get(5, 5).map(|cell| cell.symbol.as_str()), Some(" "));
}

#[test]
fn stores_complete_normalized_and_decomposed_graphemes() {
    let mut buffer = Buffer::new(12, 1);
    let text = "é e\u{301} は\u{3099} ✈\u{fe0f} 👨‍👩‍👧‍👦";

    let columns = buffer.set_string(0, 0, text, Style::new());

    assert_eq!(columns, 12);
    assert_eq!(buffer.get(0, 0).unwrap().symbol, "é");
    assert_eq!(buffer.get(2, 0).unwrap().symbol, "e\u{301}");
    assert_eq!(buffer.get(4, 0).unwrap().symbol, "は\u{3099}");
    assert!(buffer.get(5, 0).unwrap().is_continuation);
    assert_eq!(buffer.get(7, 0).unwrap().symbol, "✈\u{fe0f}");
    assert!(buffer.get(8, 0).unwrap().is_continuation);
    assert_eq!(buffer.get(10, 0).unwrap().symbol, "👨‍👩‍👧‍👦");
    assert!(buffer.get(11, 0).unwrap().is_continuation);
}

#[test]
fn clips_only_at_grapheme_boundaries() {
    let mut buffer = Buffer::new(4, 1);
    buffer.set_string(0, 0, "keep", Style::new());

    assert_eq!(buffer.set_string(3, 0, "界", Style::new()), 0);
    assert_eq!(buffer.get(3, 0).unwrap().symbol, "p");
    assert!(!buffer.get(3, 0).unwrap().is_continuation);

    assert_eq!(buffer.set_string(3, 0, "e\u{301}", Style::new()), 1);
    assert_eq!(buffer.get(3, 0).unwrap().symbol, "e\u{301}");
}

#[test]
fn narrow_replacements_remove_every_stale_wide_cell() {
    let mut buffer = Buffer::new(4, 1);
    buffer.set_string(0, 0, "界", Style::new());
    buffer.set_string(0, 0, "x", Style::new());
    assert_eq!(buffer.get(0, 0).unwrap().symbol, "x");
    assert_eq!(buffer.get(1, 0).unwrap(), &Cell::EMPTY);

    buffer.set_string(0, 0, "界", Style::new());
    buffer.set_string(1, 0, "y", Style::new());
    assert_eq!(buffer.get(0, 0).unwrap(), &Cell::EMPTY);
    assert_eq!(buffer.get(1, 0).unwrap().symbol, "y");
}

#[test]
fn partial_clear_removes_the_intersected_grapheme_atomically() {
    let mut buffer = Buffer::new(4, 1);
    buffer.set_string(0, 0, "界z", Style::new());

    buffer.clear_area(Rect::new(1, 0, 1, 1));

    assert_eq!(buffer.get(0, 0).unwrap(), &Cell::EMPTY);
    assert_eq!(buffer.get(1, 0).unwrap(), &Cell::EMPTY);
    assert_eq!(buffer.get(2, 0).unwrap().symbol, "z");
}

#[test]
fn merge_preserves_complete_clusters_and_skips_clipped_wide_clusters() {
    let mut source = Buffer::new(3, 1);
    source.set_string(0, 0, "e\u{301}界", Style::new());
    let mut destination = Buffer::new(4, 1);
    destination.set_string(0, 0, "stay", Style::new());

    destination.merge(&source, 1, 0);

    assert_eq!(destination.get(1, 0).unwrap().symbol, "e\u{301}");
    assert_eq!(destination.get(2, 0).unwrap().symbol, "界");
    assert!(destination.get(3, 0).unwrap().is_continuation);

    destination.clear();
    destination.set_string(0, 0, "keep", Style::new());
    destination.merge(&source, 3, 0);
    assert_eq!(destination.get(3, 0).unwrap().symbol, "e\u{301}");
    assert!(!destination.iter().any(|(_, _, cell)| cell.is_continuation));
}

#[test]
fn diff_distinguishes_exact_nfc_and_nfd_cells_at_equal_width() {
    let mut nfc = Buffer::new(2, 1);
    let mut nfd = Buffer::new(2, 1);
    assert_eq!(nfc.set_string(0, 0, "é", Style::new()), 1);
    assert_eq!(nfd.set_string(0, 0, "e\u{301}", Style::new()), 1);

    let differences = nfc.diff(&nfd).collect::<Vec<_>>();
    assert_eq!(differences.len(), 1);
    assert_eq!((differences[0].0, differences[0].1), (0, 0));
    assert_eq!(differences[0].2.symbol, "é");
}

#[test]
fn character_writes_preserve_style_and_repair_continuations() {
    let style = Style::new().bold();
    let mut buffer = Buffer::new(3, 1);
    buffer.set_string(0, 0, "界", style);

    buffer.set_char(1, 0, 'x', None);

    assert_eq!(buffer.get(0, 0).unwrap(), &Cell::EMPTY);
    assert_eq!(buffer.get(1, 0).unwrap().symbol, "x");
    assert_eq!(buffer.get(1, 0).unwrap().style, style);
}
