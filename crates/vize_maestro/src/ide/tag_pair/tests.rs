use super::names_at;

/// Resolve the names at the caret marked with `|` and render them back into the
/// input with `[...]` around each resolved span, so a table of cases reads as
/// the markup the editor would highlight.
fn resolved(marked: &str) -> Option<String> {
    let offset = marked
        .find('|')
        .expect("test input marks the caret with `|`");
    let content = marked.replace('|', "");
    let names = names_at(&content, (0, content.len()), offset)?;

    let mut rendered = String::with_capacity(content.len() + 4);
    let mut cursor = 0;
    for (start, end) in [Some(names.first), names.second].into_iter().flatten() {
        rendered.push_str(&content[cursor..start]);
        rendered.push('[');
        rendered.push_str(&content[start..end]);
        rendered.push(']');
        cursor = end;
    }
    rendered.push_str(&content[cursor..]);
    Some(rendered)
}

#[test]
fn resolves_pairs_and_recovers_from_malformed_markup() {
    let cases = [
        // The pair resolves from either side of the element.
        ("<di|v>x</div>", Some("<[div]>x</[div]>")),
        ("<div>x</di|v>", Some("<[div]>x</[div]>")),
        // Nesting is resolved by stack depth, never by name.
        (
            "<div><di|v>x</div></div>",
            Some("<div><[div]>x</[div]></div>"),
        ),
        // An unclosed inner element does not hide the enclosing pair.
        ("<di|v><span>x</div>", Some("<[div]><span>x</[div]>")),
        // Elements with no second name to keep in sync.
        ("<b|r>", Some("<[br]>")),
        ("<di|v />", Some("<[div] />")),
        ("<di|v>x", Some("<[div]>x")),
        ("x</di|v>", Some("x</[div]>")),
        // Comments are text, not markup.
        (
            "<!-- <div> --><di|v>x</div>",
            Some("<!-- <div> --><[div]>x</[div]>"),
        ),
        // An unterminated start tag keeps its own name ...
        ("<div>x</div><sp|a", Some("<div>x</div><[spa]")),
        // ... and never discards what was already resolved before it.
        ("<di|v>x<spa", Some("<[div]>x<spa")),
        ("<di|v>x</div><spa", Some("<[div]>x</[div]><spa")),
        // A bare `<` in text is not a tag, with or without a `>` after it.
        ("<di|v>a < b</div>", Some("<[div]>a < b</[div]>")),
        ("<di|v>x</div>a < b", Some("<[div]>x</[div]>a < b")),
        // A close tag with no name yet has no span to highlight.
        ("<div>x</|", None),
        ("<div>x</| >", None),
        // The cursor is not on a tag name at all.
        ("<div>|x</div>", None),
    ];

    for (marked, expected) in cases {
        assert_eq!(resolved(marked).as_deref(), expected, "input: {marked}");
    }
}
