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

#[test]
fn initialization_errors_keep_the_entire_original_cause_and_resolved_path() {
    use super::errors::initialization_error;
    use std::path::Path;
    assert_eq!(
        initialization_error(
            Path::new("/controlled/runtime/tsc"),
            "updateSnapshot timed out after 30000ms"
        ),
        "Failed to start corsa type-aware session: updateSnapshot timed out after 30000ms [resolved executable: /controlled/runtime/tsc]"
    );
    assert_eq!(
        initialization_error(
            Path::new(r"C:\controlled\雪🌸\tsc.exe"),
            "unclassified cause\nsecond line"
        ),
        "Failed to start corsa type-aware session: unclassified cause\nsecond line [resolved executable: C:\\controlled\\雪🌸\\tsc.exe]"
    );
    assert_eq!(
        initialization_error(Path::new("/controlled/runtime/tsc"), ""),
        "Failed to start corsa type-aware session [resolved executable: /controlled/runtime/tsc]"
    );
}
