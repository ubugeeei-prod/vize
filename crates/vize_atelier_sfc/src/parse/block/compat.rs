pub(super) fn can_start_string_literal(prev_significant_char: u8, quote: u8) -> bool {
    matches!(
        prev_significant_char,
        b'=' | b'('
            | b'['
            | b','
            | b':'
            | b'{'
            | b';'
            | b'\n'
            | b'?'
            | b'&'
            | b'|'
            | b'+'
            | b'-'
            | b'*'
            | b'!'
            | b'>'
            | b'<'
            | b'%'
            | b'^'
    ) || prev_significant_char.is_ascii_alphabetic()
        || (quote == b'`'
            && (prev_significant_char.is_ascii_alphanumeric()
                || prev_significant_char == b'_'
                || prev_significant_char == b')'))
}

pub(super) fn is_void_block(tag_name: &[u8]) -> bool {
    std::str::from_utf8(tag_name).is_ok_and(vize_carton::is_void_tag)
}
