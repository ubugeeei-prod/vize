use super::import_specifiers;

fn legacy_import_specifiers(content: &str) -> Vec<&str> {
    let mut specifiers = Vec::new();
    for token in ["from ", "import(", "import ", "require("] {
        for (at, _) in content.match_indices(token) {
            let rest = content
                .get(at + token.len()..)
                .unwrap_or_default()
                .trim_start();
            let Some((quote, rest)) = ['\'', '"']
                .into_iter()
                .find_map(|quote| Some((quote, rest.strip_prefix(quote)?)))
            else {
                continue;
            };
            let Some((specifier, _)) = rest.split_once(quote) else {
                continue;
            };
            specifiers.push(specifier);
        }
    }
    specifiers
}

#[test]
fn preserves_legacy_token_priority_and_lexical_matches() {
    let source = concat!(
        "require('./last'); ",
        "import './third'; ",
        "import(\"./second\"); ",
        "export { value } from \t'./first'; ",
        "notfrom './still-found'; ",
        "require(\u{2003}\"./unicode-space\");",
    );

    assert_eq!(
        import_specifiers(source),
        vec![
            "./first",
            "./still-found",
            "./second",
            "./third",
            "./last",
            "./unicode-space",
        ]
    );
}

#[test]
fn matches_legacy_scanner_across_mixed_inputs() {
    const CHUNKS: &[&str] = &[
        "from ",
        "import(",
        "import ",
        "require(",
        "fromx ",
        "imported ",
        "required(",
        "'",
        "\"",
        "./local",
        "../parent",
        "package",
        " ",
        "\t",
        "\n",
        "\u{2003}",
        ")",
        ";",
        "x",
        "é",
    ];

    let mut state = 0x9e37_79b9_u32;
    for case in 0..512 {
        let mut source = std::string::String::new();
        for _ in 0..48 {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            source.push_str(CHUNKS[state as usize % CHUNKS.len()]);
        }

        assert_eq!(
            import_specifiers(&source),
            legacy_import_specifiers(&source),
            "scanner mismatch in generated case {case}: {source:?}"
        );
    }
}
