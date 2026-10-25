use super::{CssVBindEdit, copied_origin};

#[test]
fn only_actual_copied_intervals_can_recover_an_authored_offset() {
    let edits = [
        CssVBindEdit {
            source: 4..6,
            generated: 4..8,
        },
        CssVBindEdit {
            source: 7..9,
            generated: 9..13,
        },
    ];
    for (byte, expected) in [
        (0, Some(0)),
        (3, Some(3)),
        (4, None),
        (7, None),
        (8, Some(6)),
        (9, None),
        (12, None),
        (13, Some(9)),
        (14, Some(10)),
    ] {
        assert_eq!(copied_origin(byte, &edits), expected);
    }
    let adjacent = [
        CssVBindEdit {
            source: 0..2,
            generated: 0..5,
        },
        CssVBindEdit {
            source: 2..4,
            generated: 5..8,
        },
    ];
    for byte in 0..8 {
        assert_eq!(copied_origin(byte, &adjacent), None);
    }
    assert_eq!(copied_origin(8, &adjacent), Some(4));
}
