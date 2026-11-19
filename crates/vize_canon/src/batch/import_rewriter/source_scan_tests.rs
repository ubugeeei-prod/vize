use super::source_may_contain_relative_specifier;

fn legacy_source_may_contain_relative_specifier(source: &str) -> bool {
    [
        "'./", "\"./", "'../", "\"../", "'.'", "\".\"", "'..'", "\"..\"",
    ]
    .iter()
    .any(|needle| source.contains(needle))
}

#[test]
fn recognizes_the_legacy_relative_literal_shapes() {
    for source in [
        "import './child'",
        "import \"./child\"",
        "import '../parent'",
        "import \"../parent\"",
        "import '.'",
        "import \".\"",
        "import '..'",
        "import \"..\"",
        "é'import('../child')",
    ] {
        assert!(source_may_contain_relative_specifier(source), "{source:?}");
    }

    for source in [
        "import 'package'",
        "const mismatched = '.\"",
        "const parent = '..suffix'",
        "const slash = '/absolute'",
        "é😀",
    ] {
        assert!(!source_may_contain_relative_specifier(source), "{source:?}");
    }
}

#[test]
fn matches_the_legacy_scan_exhaustively_around_quotes_and_dots() {
    const ALPHABET: &[u8] = b"'\"./x";

    for len in 0..=7 {
        let cases = ALPHABET.len().pow(len);
        for mut encoded in 0..cases {
            let mut source = std::string::String::with_capacity(len as usize);
            for _ in 0..len {
                source.push(ALPHABET[encoded % ALPHABET.len()] as char);
                encoded /= ALPHABET.len();
            }
            assert_eq!(
                source_may_contain_relative_specifier(&source),
                legacy_source_may_contain_relative_specifier(&source),
                "scanner mismatch for {source:?}"
            );
        }
    }
}
