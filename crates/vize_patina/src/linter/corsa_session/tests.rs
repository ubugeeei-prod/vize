use super::{paths::next_session_directory_name, probe::byte_offset_to_utf16_offset};

#[test]
fn utf16_offsets_follow_typescript_rules() {
    assert_eq!(byte_offset_to_utf16_offset("hello", 5), 5);
    assert_eq!(byte_offset_to_utf16_offset("a😀b", 1), 1);
    assert_eq!(byte_offset_to_utf16_offset("a😀b", 5), 3);
}

#[test]
fn session_directory_names_are_unique() {
    let first = next_session_directory_name();
    let second = next_session_directory_name();
    assert_ne!(first, second);
}
