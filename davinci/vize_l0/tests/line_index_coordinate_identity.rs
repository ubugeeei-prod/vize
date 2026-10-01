//! Coordinate migration keeps the existing host-facing value identity.
use vize_l0::line_index::{LineIndex, Position, Range, offset_to_position};

#[test]
fn indexed_positions_keep_the_legacy_host_identity_and_json() {
    fn host_position(position: vize_l0::lsp::Position) -> Position {
        position
    }
    fn host_range(range: vize_l0::lsp::Range) -> Range {
        range
    }

    let source = "é😀\r\nx";
    let position = host_position(LineIndex::new(source).position("é😀\r\n".len()));
    assert_eq!(position, Position::new(1, 0));
    assert_eq!(offset_to_position(source, "é😀".len()), Position::new(0, 3));
    assert_eq!(
        serde_json::to_string(&position).unwrap(),
        r#"{"line":1,"character":0}"#
    );

    let range = host_range(Range::from_positions(0, 1, 0, 3));
    assert_eq!(range, Range::new(Position::new(0, 1), Position::new(0, 3)));
    assert_eq!(
        serde_json::to_string(&range).unwrap(),
        r#"{"start":{"line":0,"character":1},"end":{"line":0,"character":3}}"#,
    );
    assert_eq!(
        serde_json::from_str::<Range>(
            r#"{"start":{"line":0,"character":1},"end":{"line":0,"character":3}}"#
        )
        .unwrap(),
        range
    );
}
