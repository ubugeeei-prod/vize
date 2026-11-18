//! Code points HTML's tokenizer reports wherever they appear in the input.

use vize_l0::{ErrorCode, Span};

/// Each control code point other than ASCII whitespace and U+0000 (which
/// Vue reports itself), as HTML's `control-character-in-input-stream`.
pub(super) fn control_characters(source: &str, report: &mut impl FnMut(ErrorCode, Span)) {
    // Every such code point is in U+0001..=U+009F, so most bytes fail fast.
    if !source
        .bytes()
        .any(|byte| byte < b' ' || byte == 0x7F || byte == 0xC2)
    {
        return;
    }
    for (start, character) in source.char_indices() {
        let control = matches!(character, '\u{1}'..='\u{1F}' | '\u{7F}'..='\u{9F}')
            && !matches!(character, '\t' | '\n' | '\u{C}' | '\r');
        if control {
            let end = start + character.len_utf8();
            report(
                ErrorCode::ControlCharacterInInputStream,
                Span::new(start as u32, end as u32),
            );
        }
    }
}
