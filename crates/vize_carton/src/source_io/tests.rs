use super::read_to_string;

#[test]
fn file_reader_preserves_contents_and_standard_error_contracts() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("source.vue");
    for contents in ["", "\u{feff}<template>日本語🦀</template>\r\n", "\0"].iter() {
        std::fs::write(&path, contents).unwrap();
        assert_eq!(
            read_to_string(&path).unwrap(),
            std::fs::read_to_string(&path).unwrap()
        );
    }
    let mut invalid = vec![b'a'; 65536];
    invalid.extend([0xf0, 0x9f]);
    std::fs::write(&path, invalid).unwrap();
    let expected = std::fs::read_to_string(&path).unwrap_err();
    let actual = read_to_string(&path).unwrap_err();
    assert_eq!(actual.kind(), expected.kind());
    assert_eq!(actual.to_string(), expected.to_string());
    let missing = directory.path().join("missing.vue");
    let expected = std::fs::read_to_string(&missing).unwrap_err();
    let actual = read_to_string(&missing).unwrap_err();
    assert_eq!(actual.kind(), expected.kind());
    assert_eq!(actual.raw_os_error(), expected.raw_os_error());
    assert_eq!(actual.to_string(), expected.to_string());
}
