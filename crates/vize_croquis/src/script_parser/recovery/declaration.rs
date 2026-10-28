//! Declaration boundary used by incomplete-call recovery.

pub(super) fn starts_declaration(source: &str) -> bool {
    [
        "const",
        "let",
        "var",
        "function",
        "class",
        "interface",
        "type",
        "enum",
        "import",
        "export",
    ]
    .into_iter()
    .any(|keyword| {
        source.strip_prefix(keyword).is_some_and(|rest| {
            rest.chars().next().is_some_and(|character| {
                character.is_whitespace() || matches!(character, '{' | '*')
            })
        })
    })
}
