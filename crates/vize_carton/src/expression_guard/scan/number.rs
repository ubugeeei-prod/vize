pub fn skip_number(bytes: &[u8], i: usize) -> usize {
    i + bytes.get(i..).map_or(0, |rest| {
        rest.iter()
            .take_while(
                |byte| matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'.'),
            )
            .count()
    })
}

pub fn keyword_allows_regex_after(identifier: &[u8]) -> bool {
    matches!(
        identifier,
        b"await"
            | b"case"
            | b"delete"
            | b"do"
            | b"else"
            | b"in"
            | b"instanceof"
            | b"new"
            | b"of"
            | b"return"
            | b"throw"
            | b"typeof"
            | b"void"
            | b"yield"
    )
}
