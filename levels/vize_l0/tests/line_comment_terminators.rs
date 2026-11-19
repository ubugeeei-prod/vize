//! ECMAScript line-comment boundaries shared by expression and SFC scanners.

use vize_l0::expression_guard::scan::{line_comment_terminator, skip_line_comment};

#[test]
fn every_terminator_has_its_complete_byte_range() {
    for (source, expected) in [
        ("// note\nnext", 7..8),
        ("// note\rnext", 7..8),
        ("// note\r\nnext", 7..9),
        ("// note\u{2028}next", 7..10),
        ("// note\u{2029}next", 7..10),
        ("雪// 😀\u{2028}next", 10..13),
    ] {
        assert_eq!(line_comment_terminator(source.as_bytes(), 2), expected);
        assert_eq!(skip_line_comment(source.as_bytes(), 2), expected.start);
    }
}

#[test]
fn other_unicode_bytes_do_not_end_a_comment() {
    let source = "// €\u{202f}\u{0085}雪😀\u{2029}next";
    assert_eq!(line_comment_terminator(source.as_bytes(), 2), 18..21);
    assert_eq!(skip_line_comment(source.as_bytes(), 2), 18);
    for source in [b"// no terminator".as_slice(), b"// \xe2", b"// \xe2\x80"] {
        assert_eq!(
            line_comment_terminator(source, 2),
            source.len()..source.len()
        );
        assert_eq!(skip_line_comment(source, 2), source.len());
    }
}

#[test]
fn caller_offsets_and_eof_are_preserved() {
    let source = b"// first\n// second\r\nnext";
    assert_eq!(line_comment_terminator(source, 11), 18..20);
    for start in [source.len(), source.len() + 1] {
        assert_eq!(line_comment_terminator(source, start), start..start);
        assert_eq!(skip_line_comment(source, start), start);
    }
    let source = [vec![b'x'; 32_768], b"\xe2\x80\xa8next".to_vec()].concat();
    assert_eq!(line_comment_terminator(&source, 0), 32_768..32_771);
}
