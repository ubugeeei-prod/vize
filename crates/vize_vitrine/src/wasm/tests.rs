use super::utf8_byte_to_utf16_offset;

#[test]
fn test_utf8_byte_to_utf16_offset_handles_multibyte_and_astral_chars() {
    let source = "aあ😀b";

    let hiragana_start = source.find('あ').expect("hiragana should exist") as u32;
    let emoji_start = source.find('😀').expect("emoji should exist") as u32;
    let latin_b_start = source.find('b').expect("latin b should exist") as u32;

    assert_eq!(utf8_byte_to_utf16_offset(source, 0), 0);
    assert_eq!(utf8_byte_to_utf16_offset(source, hiragana_start), 1);
    assert_eq!(utf8_byte_to_utf16_offset(source, emoji_start), 2);
    assert_eq!(utf8_byte_to_utf16_offset(source, latin_b_start), 4);
    assert_eq!(utf8_byte_to_utf16_offset(source, source.len() as u32), 5);
}
